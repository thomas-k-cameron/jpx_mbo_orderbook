
macro_rules! callback_many {
    ($( $cb:path )+) => {
        $( $cb )+ 
    };
}

struct Variables {
    message_count: usize,
    key_count: usize,
    ts: Option<NaiveDateTime>,
}

#[derive(Default)]
struct PerTimeframe {
    changes: HashSet<i64>,
    executions: Vec<OrderExecution>,
    ctag_with_ptag: Vec<CTagWithCorrespondingPTag>,
    deletion: Vec<OrderDeletion>,
    modified_order_id_map: HashMap<UniqueId, (Option<Box<AddOrder>>, Option<Box<DeleteOrder>>, Option<AddOrder>)>,
    second_messages: Vec<SecondTag>
}

fn process_msg(
    order_book_map: &mut HashMap<i64, OrderBook>,
    timestamp: NaiveDateTime,
    stack: Vec<MessageEnum>,
    var: &mut Variables,
    tf: &mut PerTimeframe
) {
    
    var.message_count += stack.len();
    var.key_count += 1;

    var.ts.replace(timestamp);
    tf.changes.clear();
    // sort stack
    // stacks put order retrieved after `Executed` message  is handled
    let mut executions = &mut tf.executions;
    // stacks put order retrieved after `ExecutionWithPriceInfo` message is handled
    let mut executed_with_price_info = &mut tf.ctag_with_ptag;
    // stacks put order retrieved after `DeleteOrder` message is handled
    let mut deletion = &mut tf.deletion;

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

        let mut modified_orders_map = HashMap::new();
        for id in add_set.intersection(&del_set) {
            modified_orders_map.insert(*id, (None, None, None));
        }
        modified_orders_map
    };

    let mut second_messages = vec![];

    for msg in stack.clone() {
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
                    .expect(&err_msg(msg.order_book_id, &msg))
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
                tf.changes.insert(msg.order_book_id);
                let id = (&*msg).try_into().unwrap();
                if modified_order_id_map.contains_key(&id) {
                    modified_order_id_map.entry(id).and_modify(|opts| {
                        opts.0.replace(msg.clone());
                    });
                }
                order_book_map
                    .get_mut(&msg.order_book_id)
                    .expect(&err_msg(msg.order_book_id, &msg))
                    .add(*msg);
            }
            MessageEnum::DeleteOrder(msg) => {
                tf.changes.insert(msg.order_book_id);
                // original add order
                let add_order = order_book_map
                    .get_mut(&msg.order_book_id)
                    .expect(&err_msg(msg.order_book_id, &msg))
                    .delete(&msg);

                // modify
                let id = (&*msg).try_into().unwrap();
                if modified_order_id_map.contains_key(&id) {
                    modified_order_id_map.entry(id).and_modify(|opts| {
                        opts.1.replace(msg);
                        opts.2.replace(add_order);
                    });
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
                tf.changes.insert(msg.order_book_id);
                let add_order = order_book_map
                    .get_mut(&msg.order_book_id)
                    .expect(&err_msg(msg.order_book_id, &msg))
                    .executed(&msg);
                let item = OrderExecution {
                    matched_order_after_execution: add_order,
                    msg: *msg,
                };
                executions.push(item);
            }
            MessageEnum::ExecutionWithPriceInfo(msg) => {
                tf.changes.insert(msg.order_book_id);
                let add_order = order_book_map
                    .get_mut(&msg.order_book_id)
                    .expect(&err_msg(msg.order_book_id, &msg))
                    .c_executed(&msg);

                'a: {
                    for i in executed_with_price_info.iter_mut() {
                        if i.c_tag.match_id == msg.match_id {
                            i.paired_ctag.replace(*msg);
                            i.matched_add_order2.replace(add_order);
                            break 'a;
                        }
                    }
                    executed_with_price_info.push(CTagWithCorrespondingPTag {
                        c_tag: *msg,
                        paired_ctag: None,
                        p_tags: Vec::with_capacity(2),
                        matched_add_order: add_order,
                        matched_add_order2: None,
                    });
                };
            }
            // things that I don't know what to do with
            MessageEnum::CombinationProduct(msg) => {
                if let Some(book) = order_book_map.get_mut(&msg.combination_order_book_id) {
                    book.push_combination_orderbook(*msg)
                } else {
                    unreachable!("{} => {:?}", msg.combination_order_book_id, msg);
                };
            }
            MessageEnum::LegPrice(msg) => 'a: {
                for i in executed_with_price_info.iter_mut() {
                    if msg.combo_group_id == i.c_tag.combo_group_id {
                        i.p_tags.push(*msg);
                        break 'a;
                    }
                }
                unreachable!("CTagWithCorrespondingPTag not found for LegPrice {msg:#?}\n");
            }
            MessageEnum::SystemEventInfo(_msg) => {
                //
            }
        };
    }


    std::mem::take(&mut tf);
}
