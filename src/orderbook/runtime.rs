use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    io,
};

use chrono::NaiveDateTime;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

use crate::{datatypes::*, OrderBook};

pub trait OrderBookRunTimeCallback {
    /// executes
    #[inline]
    fn pre_message(&mut self, order_book_map: &mut HashMap<u64, OrderBook>, msg: &MessageEnum) {}
    #[inline]
    fn after_message(&mut self, order_book_map: &mut HashMap<u64, OrderBook>) {}
    #[inline]
    fn timeframe_start(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
        stack: &[MessageEnum],
    ) {
    }
    #[inline]
    fn timeframe_end(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
    ) {
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

    for (timestamp, stack) in key_as_timestamp {
        callback.timeframe_start(order_book_map, &timestamp, &stack[..]);
        // pre processing
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
                MessageEnum::PutOrder(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .put(msg);
                }
                MessageEnum::DeleteOrder(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .delete(&msg);
                }
                MessageEnum::Executed(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .executed(&msg);
                }
                MessageEnum::ExecutionWithPriceInfo(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .c_executed(&msg);
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
        // post processing
        callback.timeframe_end(order_book_map, &timestamp);
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
