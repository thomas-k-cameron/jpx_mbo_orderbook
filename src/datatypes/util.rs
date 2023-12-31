use std::str::{
    self,
    FromStr,
};
use chrono::NaiveDateTime;

use crate::OrderBook;

pub fn extract_value<'a>(s: &'a str) -> Option<&'a str> {
    if let Some((a, b)) = s.find("(").zip(s.find(")")) {
        return Some(&s[a + 1..b]);
    }
    None
}

pub fn extract_value_and_parse<T: FromStr>(s: &str) -> Option<T> {
    if let Ok(t) = FromStr::from_str(s) {
        Some(t)
    } else if let Some((a, b)) = s.find("(").zip(s.find(")")) {
        (&s[(a + 1)..b]).parse().ok()
    } else {
        None
    }
}

pub fn extract_datetime_string(s: &str) -> Option<&str> {
    if let Some(a) = s.find("(") {
        return Some(&s[..a]);
    }
    None
}

pub fn extract_datetime<'a>(s: &'a str) -> Option<NaiveDateTime> {
    if let Some(a) = s.find("(") {
        return NaiveDateTime::parse_from_str(&s[..a], "%Y-%m-%dT%H:%M:%S%.9f").ok();
    }
    None
}

pub fn ticks_between_price(book: &OrderBook, price_1: i64, price_2: i64) -> Option<i64> {
    if price_1 == price_2 {
        return Some(0);
    }

    let mut arr = [price_1, price_2];
    arr.sort();

    let mut rev_tick = (None, None);
    for i in book.tick_info.iter() {
        let r = i.price_from..i.price_to;
        if r.contains(&arr[0]) {
            rev_tick.0.replace(i);
        }
        if r.contains(&arr[1]) {
            rev_tick.1.replace(i);
        }
    }

    let (a, b) = rev_tick;
    match (a, b) {
        (Some(a), Some(b)) => {
            if a.price_from == b.price_from && a.price_to == b.price_to {
                return Some((arr[1] - arr[0]) / a.tick_size);
            } else {
                let lower_range = a.price_to - arr[0];
                let upper_range = arr[1] - b.price_from;
                let count1 = lower_range / a.tick_size;
                let count2 = upper_range / a.tick_size;
                return Some(count1 + count2);
            }
        }
        _ => return None,
    }
}

pub fn is_out_of_tick_range(book: &OrderBook, price: i64) -> bool {
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    for i in book.tick_info.iter() {
        min = if min > i.price_from {
            i.price_from
        } else {
            min
        };
        max = if max < i.price_to { i.price_to } else { max };
    }
    (min..=max).contains(&price)
}

///
/// ```rust
/// let c = $to_iter.chars().next();
/// match c {
///     Some(i) if $tag == i => (),
///     _ => return Err(()),
/// };
/// ```
#[macro_export]
macro_rules! tag_guard {
    ($tag:literal, $to_iter:ident) => {
        let c = $to_iter.chars().next();
        match c {
            Some(i) if $tag == i => (),
            _ => return Err(()),
        };
    };
}
