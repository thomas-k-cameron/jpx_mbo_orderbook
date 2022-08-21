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
