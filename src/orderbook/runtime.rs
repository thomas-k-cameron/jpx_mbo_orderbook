use std::collections::{
    HashMap,
    HashSet,
};
use std::fmt::Debug;
use std::time::{
    Duration,
    SystemTime,
};

use chrono::NaiveDateTime;

use crate::callback_datatype::*;
use crate::datatypes::*;
use crate::{
    MessageEnum,
    OrderBook,
};

pub trait OrderBookRunTimeCallback {
    // stops the runtime when true is returned
    /// Called before event_start and before processing a message.
    /// Stops processing any data once true is returned.
    #[inline(always)]
    fn stop(&mut self) -> bool {
        false
    }

    /// Called before processing the message stack.
    #[allow(unused_variables)]
    #[inline]
    fn event_start(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        stack: &[MessageEnum],
    ) {
    }
    /// called after processing of the message stack is done.
    #[allow(unused_variables)]
    #[inline]
    fn event_end(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        stack: &[MessageEnum],
    ) {
    }

    /// called if a message stack contains T tag.
    #[allow(unused_variables)]
    #[inline]
    fn second_message(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        second_messages: &[SecondTag],
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    fn created(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        created: Created,
    ) {
    }

    /// called if there was an A, E or D tag message in the message stack
    #[allow(unused_variables)]
    #[inline]
    fn order_book_id_with_changes(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        changes: &HashSet<i64>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called only if `E` tag was in the message stack
    fn executions(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        executions: Vec<OrderExecution>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called only if `C` tag was in the message stack
    fn ctag_execution(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        executed_with_price_info: Vec<CTagWithCorrespondingPTag>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called only if `D` tag was in the message stack
    fn deletions(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        deletion: Vec<OrderDeletion>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// called when order(s) are modified  
    /// called if message stack has D tag message and A tag message with same unique_id
    fn modified_orders(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: &NaiveDateTime,
        modified_orders: Vec<ModifiedOrder>,
    ) {
    }

    #[allow(unused_variables)]
    #[inline]
    /// Called when there are no messages left or stop returned true.
    fn all_done(
        &mut self,
        order_book_map: &HashMap<i64, OrderBook>,
        timestamp: Option<NaiveDateTime>,
    ) {
    }
}

pub struct RuntimeStats {
    pub message_count: usize,
    pub key_count: usize,
    pub time_taken: Duration,
}

pub fn order_book_runtime<A>(
    order_book_map: &mut HashMap<i64, OrderBook>,
    mut key_as_timestamp: impl Iterator<Item = (NaiveDateTime, Vec<MessageEnum>)>,
    callback: &mut A,
) -> RuntimeStats
where
    A: OrderBookRunTimeCallback,
{
    fn err_msg(order_book_id: i64, message: impl Debug) -> String {
        format!(
            "error when retreiving {}: MessageSnapshot: {:?}",
            order_book_id, message
        )
    }

    let mut ts = None;
    let mut message_count = 0;
    let mut key_count = 0;
    let now = SystemTime::now();
    'outer: while let Some((timestamp, stack)) = key_as_timestamp.next() {
        // list of all order book id who had something changes to price levels
        let mut changes = HashSet::new();
        // stacks put order retrieved after `Executed` message  is handled
        let mut executions = vec![];
        // stacks put order retrieved after `ExecutionWithPriceInfo` message is handled
        let mut executed_with_price_info: Vec<CTagWithCorrespondingPTag> = vec![];
        // stacks put order retrieved after `DeleteOrder` message is handled
        let mut deletion = vec![];
        // stacks newly created orders
        let mut created = vec![];
        if (callback.stop()) {
            break 'outer;
        }

        message_count += stack.len();
        key_count += 1;

        ts.replace(timestamp);
        changes.clear();
        // sort stack
        callback.event_start(order_book_map, &timestamp, &stack[..]);
        // pre processing

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

            let mut modified_orders_map = HashMap::with_capacity(del_set.len());
            for id in add_set.intersection(&del_set) {
                modified_orders_map.insert(*id, (None, None, None));
            }

            modified_orders_map
        };

        let mut second_messages = vec![];

        for msg in stack.clone() {
            if (callback.stop()) {
                break 'outer;
            }

            match msg {
                MessageEnum::SecondTag(msg) => {
                    second_messages.push(*msg);
                }
                // this one creates order book
                MessageEnum::ProductInfo(info) => {
                    let order_book_id = info.order_book_id;
                    let check = order_book_map.insert(order_book_id, OrderBook::new(*info));
                    match check {
                        Some(ob) => {
                            // check if the product_info is pointing at the same instrument
                            let mut i1 = ob.product_info.clone();
                            let mut i2 = order_book_map
                                .get(&ob.order_book_id())
                                .unwrap()
                                .product_info
                                .clone();
                            i1.timestamp = NaiveDateTime::MAX;
                            i2.timestamp = NaiveDateTime::MAX;
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
                        book.push_trading_status(*msg);
                    } else {
                        println!("{}", err_msg(msg.order_book_id, &msg));
                    };
                }
                MessageEnum::TickSize(msg) => {
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .unwrap_or_else(|| unreachable!("{}", err_msg(msg.order_book_id, &msg)))
                        .append_l(*msg);
                }
                MessageEnum::EquilibriumPrice(msg) => {
                    let check = order_book_map.get_mut(&msg.order_book_id);

                    if let Some(book) = check {
                        book.push_last_equilibrium_price(*msg);
                    } else {
                        eprintln!("{}", err_msg(msg.order_book_id, &msg));
                    }
                }
                // order CRUD. New order insertion, deletion, execution (reduction of order qty)
                MessageEnum::AddOrder(msg) => {
                    changes.insert(msg.order_book_id);
                    let id = (&*msg).try_into().unwrap();
                    if let Some(opts) = modified_order_id_map.get_mut(&id) {
                        opts.0.replace(msg.clone());
                    } else {
                        created.push(*msg);
                    };
                    order_book_map
                        .get_mut(&msg.order_book_id)
                        .unwrap_or_else(|| unreachable!("{}", (&err_msg(msg.order_book_id, &msg))))
                        .add(*msg);
                }
                MessageEnum::DeleteOrder(msg) => {
                    changes.insert(msg.order_book_id);
                    // original add order
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .unwrap_or_else(|| unreachable!("{}", (&err_msg(msg.order_book_id, &msg))))
                        .delete(&msg);

                    // modify
                    let id = (&*msg).try_into().unwrap();
                    if let Some(opts) = modified_order_id_map.get_mut(&id) {
                        opts.1.replace(msg);
                        opts.2.replace(add_order);
                    } else {
                        // deletion
                        let item = OrderDeletion {
                            deleted_order: add_order,
                            msg: *msg,
                        };
                        deletion.push(item);
                    }
                }
                MessageEnum::Executed(msg) => {
                    changes.insert(msg.order_book_id);
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .unwrap_or_else(|| unreachable!("{}", (&err_msg(msg.order_book_id, &msg))))
                        .executed(&msg);
                    let item = OrderExecution {
                        matched_order_after_execution: add_order,
                        msg: *msg,
                    };
                    executions.push(item);
                }
                MessageEnum::ExecutionWithPriceInfo(msg) => {
                    changes.insert(msg.order_book_id);
                    let add_order = order_book_map
                        .get_mut(&msg.order_book_id)
                        .unwrap_or_else(|| unreachable!("{}", (&err_msg(msg.order_book_id, &msg))))
                        .c_executed(&msg);

                    'a: {
                        for i in executed_with_price_info.iter_mut() {
                            if i.combo_group_id == msg.combo_group_id {
                                i.c_tag.push((*msg).clone());
                                i.matched_add_order.push(add_order);
                                break 'a;
                            }
                        }
                        executed_with_price_info.push(CTagWithCorrespondingPTag {
                            combo_group_id: msg.combo_group_id,
                            c_tag: vec![(*msg).clone()],
                            matched_add_order: vec![add_order],
                            p_tags: Vec::with_capacity(2),
                        });
                    };
                }
                // things that I don't know what to do with
                MessageEnum::CombinationProduct(msg) => {
                    if let Some(book) = order_book_map.get_mut(&msg.combination_order_book_id) {
                        book.push_combination_orderbook(*msg)
                    } else {
                        //unreachable!("{} => {:?}", msg.combination_order_book_id, msg);
                    };
                }
                MessageEnum::LegPrice(msg) => 'a: {
                    for i in executed_with_price_info.iter_mut() {
                        if msg.combo_group_id == i.combo_group_id {
                            i.p_tags.push(*msg);
                            break 'a;
                        }
                    }
                    executed_with_price_info.push(CTagWithCorrespondingPTag {
                        combo_group_id: msg.combo_group_id,
                        c_tag: vec![],
                        matched_add_order: vec![],
                        p_tags: vec![*msg],
                    });
                }
                MessageEnum::SystemEventInfo(_msg) => {
                    //
                }
            };
        }

        if !second_messages.is_empty() {
            callback.second_message(&order_book_map, &timestamp, &second_messages)
        }

        if !created.is_empty() {
            callback.created(
                order_book_map,
                &timestamp,
                Created {
                    msgs: created,
                    is_fas: !(executions.is_empty() && executed_with_price_info.is_empty()),
                    executed_qty: executions
                        .iter()
                        .fold(0, |a, b| a + b.msg.executed_quantity)
                        + executed_with_price_info
                            .iter()
                            .fold(0, |a, b| a + b.executed_quantity()),
                },
            );
        }

        if !executions.is_empty() {
            callback.executions(order_book_map, &timestamp, executions);
        }

        if !executed_with_price_info.is_empty() {
            callback.ctag_execution(
                order_book_map,
                &timestamp,
                executed_with_price_info,
            );
        }

        if !deletion.is_empty() {
            callback.deletions(order_book_map, &timestamp, std::mem::take(&mut deletion));
        }

        if !modified_order_id_map.is_empty() {
            let mut modified_orders = Vec::with_capacity(modified_order_id_map.len());
            for (id, tup) in modified_order_id_map {
                match tup {
                    (Some(modify_msg), Some(delete_msg), Some(previous_add_order)) => {
                        // [減数訂正が可能であること](https://faq.sbineotrade.jp/answer/608752eba86ee343fd1372fc)
                        let modify_type = if modify_msg.quantity == previous_add_order.quantity
                            && modify_msg.price == previous_add_order.price
                        {
                            ModifyType::Neither
                        } else if modify_msg.price == previous_add_order.price {
                            ModifyType::ReduceQty
                        } else if modify_msg.quantity == previous_add_order.quantity {
                            ModifyType::PriceChange
                        } else {
                            ModifyType::Both
                        };

                        let ord = ModifiedOrder {
                            id,
                            modify_msg: *modify_msg,
                            delete_msg: *delete_msg,
                            previous_add_order,
                            modify_type,
                        };
                        modified_orders.push(ord);
                    }
                    _ => unreachable!(),
                };
            }
            callback.modified_orders(order_book_map, &timestamp, modified_orders);
        }

        if !changes.is_empty() {
            callback.order_book_id_with_changes(order_book_map, &timestamp, &changes);
        }

        // post processing
        callback.event_end(order_book_map, &timestamp, &stack[..]);
    }

    callback.all_done(order_book_map, ts);
    let time_taken = now.elapsed().unwrap();
    RuntimeStats {
        message_count,
        time_taken,
        key_count,
    }
}
