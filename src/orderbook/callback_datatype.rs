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
