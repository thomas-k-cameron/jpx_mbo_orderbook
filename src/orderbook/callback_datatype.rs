use crate::datatypes::*;

pub struct OrderExecution {
    pub matched_order_after_execution: AddOrder,
    pub msg: Executed,
}

pub struct OrderExecutionWithPriceInfo {
    pub matched_order_after_execution: AddOrder,
    pub msg: ExecutionWithPriceInfo,
}

pub struct OrderDeletion {
    /// indicates that delete tag was issued in conjunction with add order
    /// meaning that order was modified, instead of actually leaving the orderbook
    /// it went to another price level.
    pub is_order_modified: bool,
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
