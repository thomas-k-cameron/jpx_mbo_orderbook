use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

use chrono::NaiveDateTime;

use crate::MessageEnum;
use crate::{datatypes::*, OrderBook};

use crate::callback_datatype::*;

pub trait OrderBookRunTimeCallback {
    /// executes
    #[allow(unused_variables)]
    #[inline]
    fn pre_message(&mut self, order_book_map: &mut HashMap<u64, OrderBook>, msg: &MessageEnum) {}
    #[allow(unused_variables)]
    #[inline]
    fn after_message(&mut self, order_book_map: &mut HashMap<u64, OrderBook>) {}
    #[allow(unused_variables)]
    #[inline]
    fn timeframe_start(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
        stack: &[MessageEnum],
    ) {
    }
    #[allow(unused_variables)]
    #[inline]
    fn timeframe_end(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called only when `E` tag message was received
    fn executions(&mut self, timestamp: &NaiveDateTime, executions: Vec<OrderExecution>) {}

    #[allow(unused_variables)]
    #[inline]
    /// called only when `C` tag message was received
    fn executed_with_price_info(
        &mut self,
        timestamp: &NaiveDateTime,
        executed_with_price_info: Vec<OrderExecutionWithPriceInfo>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called only when `D` tag message was received
    fn deletions(&mut self, timestamp: &NaiveDateTime, deletion: Vec<OrderDeletion>) {}

    #[allow(unused_variables)]
    #[inline]
    fn all_done(&mut self, order_book_map: &mut HashMap<u64, OrderBook>, timestamp: &NaiveDateTime) {
        
    }
}

pub fn order_book_runtime<A>(
    order_book_map: &mut HashMap<u64, OrderBook>,
    key_as_timestamp: BTreeMap<NaiveDateTime, Vec<MessageEnum>>,
    callback: &mut A,
) where
    A: OrderBookRunTimeCallback,
{
    fn err_msg(order_book_id: u64, message: impl Debug) -> String {
        format!(
            "error when retreiving {}: MessageSnapshot: {:?}",
            order_book_id, message
        )
    }

    let ts = match key_as_timestamp.iter().last() {
        Some((ts, _)) => *ts,
        None => {
            println!("no message received");
            return
        }
    };
    for (timestamp, stack) in key_as_timestamp {
        // sort stack
        callback.timeframe_start(order_book_map, &timestamp, &stack[..]);
        // pre processing

        // stacks put order retrieved after `Executed` message  is handled
        let mut executions = vec![];
        // stacks put order retrieved after `ExecutionWithPriceInfo` message is handled
        let mut executed_with_price_info = vec![];
        // stacks put order retrieved after `DeleteOrder` message is handled
        let mut deletion = vec![];

        for msg in stack {
            callback.pre_message(order_book_map, &msg);
            match msg {
                MessageEnum::SecondTag(_msg) => {
                    // do nothing
                }
                // this one creates order book
                MessageEnum::ProductInfo(info) => {
                    let order_book_id = info.order_book_id;
                    order_book_map.insert(order_book_id, OrderBook::new(info));
                }
                // order book meta data update
                MessageEnum::TradingStatusInfo(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .set_trading_status(&msg);
                }
                MessageEnum::TickSize(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .append_l(msg);
                }
                MessageEnum::EquilibriumPrice(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .set_last_equilibrium_price(msg);
                }
                // order CRUD. New order insertion, deletion, execution (reduction of order qty)
                MessageEnum::AddOrder(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .put(msg);
                }
                MessageEnum::DeleteOrder(msg) => {
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .delete(&msg);

                    let item = OrderDeletion { add_order, msg };
                    deletion.push(item);
                }
                MessageEnum::Executed(msg) => {
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .executed(&msg);
                    let item = OrderExecution { add_order, msg };
                    executions.push(item);
                }
                MessageEnum::ExecutionWithPriceInfo(msg) => {
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .c_executed(&msg);

                    let item = OrderExecutionWithPriceInfo { add_order, msg };
                    executed_with_price_info.push(item);
                }
                // things that I don't know what to do with
                MessageEnum::CombinationProduct(_msg) => {
                    //msg.
                }
                MessageEnum::LegPrice(msg) => {
                    //msg.
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg));
                }
                MessageEnum::SystemEventInfo(_msg) => {
                    //
                }
            };
            callback.after_message(order_book_map);
        }

        if executions.len() > 0 {
            callback.executions(&timestamp, executions);
        }

        if executed_with_price_info.len() > 0 {
            callback.executed_with_price_info(&timestamp, executed_with_price_info);
        }

        if deletion.len() > 0 {
            callback.deletions(&timestamp, deletion);
        }

        // post processing
        callback.timeframe_end(order_book_map, &timestamp);

        // this is expected to happen only once
        if timestamp == ts {
            callback.all_done(order_book_map, &timestamp);
        }
    }
}

pub fn from_raw_file(file: String) -> ParseResult {
    let mut itch = BTreeMap::new();
    let mut unknown = vec![];
    for row in file.split("\n").map(|i| i.to_string()) {
        match MessageEnum::try_from(row) {
            Ok(i) => {
                if let Some(list) = itch.get_mut(&i.timestamp()) {
                    let list: &mut Vec<MessageEnum> = list;
                    list.push(i);
                } else {
                    itch.insert(i.timestamp(), vec![i]);
                };
            }
            Err(e) => unknown.push(e),
        }
    }

    ParseResult { itch, unknown }
}

pub struct ParseResult {
    pub itch: BTreeMap<NaiveDateTime, Vec<MessageEnum>>,
    pub unknown: Vec<String>,
}
