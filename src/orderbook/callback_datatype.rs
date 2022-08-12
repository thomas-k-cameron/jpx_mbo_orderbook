use crate::datatypes::*;

pub struct OrderExecution {
    pub put_order: AddOrder,
    pub msg: Executed,
}

pub struct OrderExecutionWithPriceInfo {
    pub put_order: AddOrder,
    pub msg: ExecutionWithPriceInfo,
}

pub struct OrderDeletion {
    pub put_order: AddOrder,
    pub msg: DeleteOrder,
}
