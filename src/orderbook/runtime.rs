use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug, path::Path,
};

use chrono::{naive, NaiveDateTime};
use tokio::{io::{BufReader, AsyncBufReadExt}, fs::File};

use crate::MessageEnum;
use crate::{datatypes::*, OrderBook};

use crate::callback_datatype::*;

pub trait OrderBookRunTimeCallback {
    // stops the runtime when true is returned
    fn stop(&mut self) -> bool {
        false
    }

    #[allow(unused_variables)]
    #[inline]
    fn event_start(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
        stack: &[MessageEnum],
    ) {
    }
    #[allow(unused_variables)]
    #[inline]
    fn event_end(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
        stack: &[MessageEnum],
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called only when `E` tag message was received
    fn executions(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
        executions: Vec<OrderExecution>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called only when `C` tag message was received
    fn executed_with_price_info(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
        executed_with_price_info: Vec<OrderExecutionWithPriceInfo>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called only when `D` tag message was received
    fn deletions(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
        deletion: Vec<OrderDeletion>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called when order(s) are modified  
    /// modified orders are detected when message with d tag and a tag refers to the same unique_id
    fn modified_orders(
        &mut self,
        order_book_map: &mut HashMap<u64, OrderBook>,
        timestamp: &NaiveDateTime,
        modified_orders: Vec<ModifiedOrder>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    fn all_done(
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

    let ts = match key_as_timestamp.iter().last() {
        Some((ts, _)) => *ts,
        None => {
            println!("no message received");
            return;
        }
    };
    for (timestamp, stack) in key_as_timestamp {
        if callback.stop() {
            break;
        }
        // sort stack
        callback.event_start(order_book_map, &timestamp, &stack[..]);
        // pre processing

        // stacks put order retrieved after `Executed` message  is handled
        let mut executions = vec![];
        // stacks put order retrieved after `ExecutionWithPriceInfo` message is handled
        let mut executed_with_price_info = vec![];
        // stacks put order retrieved after `DeleteOrder` message is handled
        let mut deletion = vec![];

        let mut modified_order_id_map = {
            let mut add_set = HashSet::new();
            let mut del_set = HashSet::new();

            for i in stack.iter() {
                match i {
                    MessageEnum::AddOrder(add) => {
                        let id = UniqueId::from_add_order(add);
                        add_set.insert(id);
                    }
                    MessageEnum::DeleteOrder(del) => {
                        let id = UniqueId::from_delete_order(del);
                        del_set.insert(id);
                    }
                    _ => (),
                }
            }

            let (longer, shorter) = if add_set.len() > del_set.len() {
                (add_set, del_set)
            } else {
                (del_set, add_set)
            };

            let mut modified_orders_map = HashMap::new();
            for id in longer {
                if shorter.contains(&id) {
                    modified_orders_map.insert(id, (None, None, None));
                }
            }
            modified_orders_map
        };

        for msg in stack.clone() {
            if callback.stop() {
                break;
            }

            match msg {
                MessageEnum::SecondTag(_msg) => {
                    // do nothing
                }
                // this one creates order book
                MessageEnum::ProductInfo(info) => {
                    let order_book_id = info.order_book_id;
                    let check = order_book_map.insert(order_book_id, OrderBook::new(info));
                    match check {
                        Some(ob) => {
                            // check if the product_info is pointing at the same instrument
                            let mut i1 = ob.product_info.clone();
                            let mut i2 = order_book_map
                                .get(&ob.order_book_id())
                                .unwrap()
                                .product_info
                                .clone();
                            i1.timestamp = naive::MAX_DATETIME;
                            i2.timestamp = naive::MAX_DATETIME;
                            // put it back if it is the same.
                            if i1 == i2 {
                                order_book_map.insert(order_book_id, ob);
                            } else {
                                unimplemented!("{:#?}", (&i1, &i2));
                            };
                        }
                        None => (), // ok!
                    }
                }
                // order book meta data update
                MessageEnum::TradingStatusInfo(msg) => {
                    let book = order_book_map.get_mut(&msg.order_book_id);

                    if let Some(book) = book {
                        book.set_trading_status(&msg);
                    } else {
                        println!("{}", err_msg(msg.order_book_id, &msg));
                    };
                }
                MessageEnum::TickSize(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .append_l(msg);
                }
                MessageEnum::EquilibriumPrice(msg) => {
                    let check = order_book_map.get_mut(&msg.order_book_id);

                    if let Some(book) = check {
                        book.set_last_equilibrium_price(msg);
                    } else {
                        eprintln!("{}", err_msg(msg.order_book_id, &msg));
                    }
                }
                // order CRUD. New order insertion, deletion, execution (reduction of order qty)
                MessageEnum::AddOrder(msg) => {
                    let id = (&msg).try_into().unwrap();
                    if modified_order_id_map.contains_key(&id) {
                        modified_order_id_map.entry(id).and_modify(|opts| {
                            opts.0.replace(msg.clone());
                        });
                    }
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .put(msg);
                }
                MessageEnum::DeleteOrder(msg) => {
                    // original add order
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .delete(&msg);

                    // modify
                    let id = (&msg).try_into().unwrap();
                    if modified_order_id_map.contains_key(&id) {
                        modified_order_id_map.entry(id).and_modify(|opts| {
                            opts.1.replace(msg.clone());
                            opts.2.replace(add_order.clone());
                        });
                    } else {
                        // deletion
                        let item = OrderDeletion {
                            deleted_order: add_order,
                            msg,
                        };
                        deletion.push(item);
                    }
                }
                MessageEnum::Executed(msg) => {
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .executed(&msg);
                    let item = OrderExecution {
                        matched_order_after_execution: add_order,
                        msg,
                    };
                    executions.push(item);
                }
                MessageEnum::ExecutionWithPriceInfo(msg) => {
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .c_executed(&msg);

                    let item = OrderExecutionWithPriceInfo {
                        matched_order_after_execution: add_order,
                        msg,
                    };
                    executed_with_price_info.push(item);
                }
                // things that I don't know what to do with
                MessageEnum::CombinationProduct(msg) => {
                    if let Some(book) = order_book_map.get_mut(&msg.combination_order_book_id) {
                        book.set_combination_orderbook(msg)
                    } else {
                        unreachable!("{} => {:?}", msg.combination_order_book_id, msg);
                    }
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
        }

        if executions.len() > 0 {
            callback.executions(order_book_map, &timestamp, executions);
        }

        if executed_with_price_info.len() > 0 {
            callback.executed_with_price_info(order_book_map, &timestamp, executed_with_price_info);
        }

        if deletion.len() > 0 {
            callback.deletions(order_book_map, &timestamp, deletion);
        }

        if modified_order_id_map.len() > 0 {
            let mut modified_orders = vec![];
            for (id, tup) in modified_order_id_map {
                let (modify_msg, delete_msg, previous_add_order) = tup;
                let (modify_msg, delete_msg, previous_add_order) = (
                    modify_msg.unwrap(),
                    delete_msg.unwrap(),
                    previous_add_order.unwrap(),
                );
                let ord = ModifiedOrder {
                    id,
                    modify_msg,
                    delete_msg,
                    previous_add_order,
                };
                modified_orders.push(ord);
            }
            callback.modified_orders(order_book_map, &timestamp, modified_orders);
        }

        // post processing
        callback.event_end(order_book_map, &timestamp, &stack[..]);
    }
    callback.all_done(order_book_map, &ts);
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

pub async fn from_filepath(filepath: impl AsRef<Path>) -> ParseResult {
    let mut itch = BTreeMap::new();
    let mut unknown = vec![];
    let mut lines = {
        let file = File::open(filepath).await.unwrap();
        BufReader::new(file).lines()
    };
    while let Ok(Some(line)) = lines.next_line().await {
        match MessageEnum::try_from(line) {
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
