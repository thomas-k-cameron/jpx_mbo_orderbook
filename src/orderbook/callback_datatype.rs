use crate::datatypes::*;

pub struct OrderExecution {
    pub add_order: AddOrder,
    pub msg: Executed,
}

pub struct OrderExecutionWithPriceInfo {
    pub add_order: AddOrder,
    pub msg: ExecutionWithPriceInfo,
}

pub struct OrderDeletion {
    pub add_order: AddOrder,
    pub msg: DeleteOrder,
}
