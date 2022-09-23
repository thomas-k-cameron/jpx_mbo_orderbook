use crate::datatypes::*;

pub struct OrderExecution {
    pub matched_order_after_execution: AddOrder,
    pub msg: Executed,
}

#[derive(Debug)]
pub struct CTagWithCorrespondingPTag {
    pub c_tag: ExecutionWithPriceInfo,
    pub matched_add_order: AddOrder,
    pub paired_ctag: Option<ExecutionWithPriceInfo>,
    pub matched_add_order2: Option<AddOrder>,
    pub p_tags: Vec<LegPrice>
}

/// struct for order that were deleted
pub struct OrderDeletion {
    pub deleted_order: AddOrder,
    pub msg: DeleteOrder,
}

pub struct ModifiedOrder {
    pub id: UniqueId,
    /// corresponding d tag
    pub delete_msg: DeleteOrder,
    /// corresponding a tag
    pub modify_msg: AddOrder,
    /// AddOrder struct removed from the orderbook
    pub previous_add_order: AddOrder,
}
