use crate::order_book_runtime;

use super::runtime::from_raw_file;
use std::collections::HashMap;

#[test]
pub fn handle() {
    let filepath = "./data/srnd-itch_20210301_A.csv";
    let file = std::fs::read_to_string(filepath).unwrap();
    let stuff = from_raw_file(file);
    let mut map = HashMap::new();
    let mut analysis = TestSpread;
    order_book_runtime(&mut map, stuff.itch, &mut analysis);
    for i in stuff.unknown {
        println!("{:?}", i);
    }
}

use crate::OrderBookRunTimeCallback;

pub struct TestSpread;

impl OrderBookRunTimeCallback for TestSpread {
    fn timeframe_end(
        &mut self,
        order_book_map: &mut std::collections::HashMap<u64, crate::OrderBook>,
        timestamp: &chrono::NaiveDateTime,
    ) {
        for (_id, book) in order_book_map {
            let mut ask_qty = None;
            let mut bid_qty = None;
            let mut ask_price = None;
            let mut bid_price = None;
            for (price, item) in book.ask.iter().next() {
                ask_qty.replace(item.iter().fold(0, |a, b| a + b.1.quantity));
                ask_price.replace(*price);
            }
            for (price, item) in book.bid.iter().next() {
                bid_qty.replace(item.iter().rev().fold(0, |a, b| a + b.1.quantity));
                bid_price.replace(*price);
            }
            assert!(
                ask_qty.is_none() || ask_qty.unwrap() > 0,
                "ask qty: timestamp: {}",
                timestamp
            );
            assert!(
                bid_qty.is_none() || bid_qty.unwrap() > 0,
                "bid qty: timestamp: {}",
                timestamp
            );
            let exist = ask_price.is_some() && bid_price.is_some();
            if exist {
                assert!(
                    ask_price.unwrap() > bid_price.unwrap(),
                    "ask: {:?}, bid: {:?}, timestamp: {} ",
                    ask_price,
                    bid_price,
                    timestamp
                )
            }
        }
    }
}
