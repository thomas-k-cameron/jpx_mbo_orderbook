use std::{collections::{HashMap, BTreeMap}, fmt::Debug, io};

use chrono::NaiveDateTime;
use tokio::io::{BufReader, AsyncBufReadExt, AsyncReadExt};

use crate::{datatypes::*, OrderBook};

pub trait OrderBookRunTimeCallback {
    /// executes 
    #[inline]
    fn pre_message(&mut self, order_book_map: &mut HashMap<u64, OrderBook>, msg: &MessageEnum) {}
    #[inline]
    fn after_message(&mut self, order_book_map: &mut HashMap<u64, OrderBook>) {}
    #[inline]
    fn timeframe_start(&mut self,  order_book_map: &mut HashMap<u64, OrderBook>, timestamp: &NaiveDateTime, stack: &[MessageEnum]) {}
    #[inline]
    fn timeframe_end(&mut self, order_book_map: &mut HashMap<u64, OrderBook>, timestamp: &NaiveDateTime) {}
}

pub async fn order_book_runtime<A>(order_book_map: &mut HashMap<u64, OrderBook>, key_as_timestamp: BTreeMap<NaiveDateTime, Vec<MessageEnum>>, analysis: &mut A) 
    where
        A: OrderBookRunTimeCallback
{
    fn err_msg(order_book_id: u64, message: impl Debug) -> String {
        format!(
            "error when retreiving {}: MessageSnapshot: {:?}",
            order_book_id, message
        )
    }
    
    for (timestamp, stack) in key_as_timestamp {
        analysis.timeframe_start(order_book_map, &timestamp, &stack[..]);
        // pre processing
        for msg in stack {
            analysis.pre_message( order_book_map, &msg);
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
                    order_book_map.get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .set_trading_status(&msg);
                }
                MessageEnum::TickSize(msg) => {
                    order_book_map.get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .append_l(msg);
                }
                MessageEnum::EquilibriumPrice(msg) => {
                    order_book_map.get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .set_last_equilibrium_price(msg);
                }
                // order CRUD. New order insertion, deletion, execution (reduction of order qty)
                MessageEnum::PutOrder(msg) => {
                    order_book_map.get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .put(msg);
                }
                MessageEnum::DeleteOrder(msg) => {
                    order_book_map.get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .delete(&msg);
                }
                MessageEnum::Executed(msg) => {
                    order_book_map.get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .executed(&msg);
                }
                MessageEnum::ExecutionWithPriceInfo(msg) => {
                    order_book_map.get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg))
                        .c_executed(&msg);
                }
                // things that I don't know what to do with
                MessageEnum::CombinationProduct(_msg) => {
                    //msg.
                }
                MessageEnum::LegPrice(msg) => {
                    //msg.
                    order_book_map.get_mut(&msg.order_book_id)
                        .expect(&err_msg(msg.order_book_id, &msg));
                }
                MessageEnum::SystemEventInfo(_msg) => {
                    //
                }
            };
            analysis.after_message(order_book_map);
        }
        // post processing
        analysis.timeframe_end(order_book_map, &timestamp);
    }
}

async fn from_raw_file(filepath: &str) -> Result<BTreeMap<NaiveDateTime, Vec<MessageEnum>>, io::Error> {
    fn msg_enum_from_raw_file_row(row: String) -> Result<MessageEnum, String> {
        if let Ok(i) = serde_json::to_value(parse_row(row)?) {
            Ok(serde_json::from_value(i).unwrap())
        } else {
            Err(row)
        }
    }

    let reader = BufReader::new(tokio::fs::File::open(filepath).await?);
    let mut dst = "".to_string();
    reader.read_to_string(&mut dst).await?;

    let mut treemap = BTreeMap::new();
    for i in dst.split("\n").map(|i| i.to_string()) {
        match msg_enum_from_raw_file_row(i) {
            Ok(i) => {
                treemap.entry(i.timestamp())
                    .and_modify(|i: &mut Vec<MessageEnum>| i.push(i))
                    .or_insert(vec![i]);
            },
            Err(e) => eprintln!("{:?}", e)
        }
    }

    Ok(treemap)
}


use parse_row::parse_row;
mod parse_row {
    use std::collections::BTreeMap;

    pub fn parse_row(s: String) -> Result<BTreeMap<String, String>, String> {
        let c = if let Some(c) = s.chars().next() {
            c
        } else {
            return Err(s);
        };

        let a = s.split(",").collect();
        let mut col_value = match c {
            'T' => {
                let mut tree = parse_t(a);
                tree.insert("timestamp".to_string(), {
                    tree.get("second").unwrap().to_string()
                });
                tree
            }
            'R' => parse_r(a),
            'M' => parse_m(a),
            'L' => parse_l(a),
            'S' => parse_s(a),
            'O' => parse_o(a),
            'A' => parse_a(a),
            'E' => parse_e(a),
            'C' => parse_c(a),
            'D' => parse_d(a),
            'P' => parse_p(a),
            'Z' => parse_z(a),
            _ => return Err(s),
        };
        col_value.insert("tag".to_string(), c.to_string());
        Ok(col_value)
    }

    macro_rules! parse_row {
        ($func_name:ident, [ $($i:tt),* ]) => {
            parse_row!(@ $func_name, [
                $(stringify!($i),)*
            ]);
        };
        (@ $func_name:ident, $tt:tt) => {
            fn $func_name(s: Vec<&str>) -> BTreeMap<String, String>
            {
                $tt.iter().zip(s.iter()).map(|(a, b)| (a.to_string(), b.to_string())).fold(
                    BTreeMap::default(),
                    |mut a, (col, value)| {
                        if !col.starts_with('_') {
                            a.insert(col, value);
                        }
                        a
                    },
                )
            }
        };
    }

    parse_row!(@ parse_t, ["_", "second"]);
    parse_row!(@ parse_r, ["_","timestamp","order_book_id","symbol","long_name","_reserved","financial_product","_trading_currency","number_of_decimal_in_price","_nominal_value","_odd_lot_size","_round_lot_size","_block_lot_size","_nominal_value","number_of_legs","underlying_order_book_id","strike_price","expiration_date","number_of_decimals_in_strike_price","put_or_call"]);
    parse_row!(
        parse_m,
        [
            _,
            timestamp,
            combination_order_book_id,
            leg_order_book_id,
            leg_side,
            leg_ratio
        ]
    );
    parse_row!(
        parse_l,
        [_, timestamp, order_book_id, tick_size, price_from, price_to]
    );
    parse_row!(parse_s, [_, timestamp, event_code]);
    parse_row!(parse_o, [_, timestamp, order_book_id, state_name]);
    parse_row!(
        parse_a,
        [
            _,
            timestamp,
            order_id,
            order_book_id,
            side,
            order_book_position,
            quantity,
            price,
            _,
            _
        ]
    );
    parse_row!(
        parse_e,
        [
            _,
            timestamp,
            order_id,
            order_book_id,
            side,
            executed_quantity,
            match_id,
            combo_group_id,
            _,
            _
        ]
    );
    parse_row!(
        parse_c,
        [
            _,
            timestamp,
            order_id,
            order_book_id,
            side,
            executed_quantity,
            match_id,
            combo_group_id,
            _,
            _,
            trade_price,
            occurred_at_cross,
            _
        ]
    );
    parse_row!(parse_d, [_, timestamp, order_id, order_book_id, side]);
    parse_row!(
        parse_p,
        [
            _,
            timestamp,
            match_id,
            combo_group_id,
            side,
            quantity,
            order_book_id,
            trade_price,
            _participant_id,
            _participant_id_counter,
            _,
            occurred_at_cross
        ]
    );
    parse_row!(
        parse_z,
        [
            _,
            timestamp,
            order_book_id,
            bid_qty_at_ep,
            ask_qty_at_ep,
            ep
        ]
    );

}