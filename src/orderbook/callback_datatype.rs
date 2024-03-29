use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::datatypes::*;

#[derive(Clone, Default)]
pub struct Created {
    pub msgs: Vec<AddOrder>,
    pub is_fas: bool,
    pub executed_qty: i64
}


#[derive(Clone)]
pub struct OrderExecution {
    pub matched_order_after_execution: AddOrder,
    pub msg: Executed,
}

#[derive(Debug, Clone)]
pub struct CTagWithCorrespondingPTag {
    pub combo_group_id: i64,
    pub c_tag: Vec<ExecutionWithPriceInfo>,
    pub matched_add_order: Vec<AddOrder>,
    pub p_tags: Vec<LegPrice>,
}
impl CTagWithCorrespondingPTag {
    pub fn order_book_id(&self) -> i64 {
        self.c_tag
            .iter()
            .next()
            .unwrap_or_else(|| panic!("{self:#?}"))
            .order_book_id
    }

    pub fn qty_by_order_book_id(&self) -> HashMap<i64, i64> {
        let mut map = HashMap::new();
        for i in self.c_tag.iter() {
            *map.entry(i.order_book_id).or_insert(0) += i.executed_quantity;
        }
        map
    }

    pub fn executed_quantity(&self) -> i64 {
        if cfg!(test) {
            self._test();
        }
        if self.c_tag.len() > 1 {
            self.c_tag
                .iter()
                .filter(|i| i.side == Side::Buy)
                .fold(0, |a, b| a + b.executed_quantity)
        } else {
            self.c_tag[0].executed_quantity
        }
    }

    pub fn occured_at_cross(&self) -> bool {
        // this function is not expected to be called when there is no item in `c_tag` field
        self.c_tag[0].occurred_at_cross
    }

    fn _test(&self) {
        let qty = self
            .c_tag
            .iter()
            .filter(|i| i.side == Side::Buy)
            .fold(0, |a, b| a + b.executed_quantity);
        let qty2 = self
            .c_tag
            .iter()
            .filter(|i| i.side == Side::Sell)
            .fold(0, |a, b| a + b.executed_quantity);
        if self.c_tag.len() > 1 {
            assert_eq!(qty, qty2);
        };
    }
}

/// struct for order that were deleted
#[derive(Debug, Clone)]
pub struct OrderDeletion {
    pub deleted_order: AddOrder,
    pub msg: DeleteOrder,
}

#[derive(Clone, Debug)]
pub struct ModifiedOrder {
    pub id: UniqueId,
    /// corresponding d tag
    pub delete_msg: DeleteOrder,
    /// corresponding a tag
    pub modify_msg: AddOrder,
    /// AddOrder struct removed from the orderbook
    pub previous_add_order: AddOrder,
    pub modify_type: ModifyType
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModifyType {
    ReduceQty,
    PriceChange,
    /// used when both `ReduceQty` and `PriceChange` is observed
    Both,
    /// used when both `ReduceQty` and `PriceChange` is not observed
    Neither
}
