use chrono::NaiveDateTime;
use std::str::{self, FromStr};

pub fn extract_value<'a>(s: &'a str) -> Option<&'a str> {
    for (a, b) in s.find("(").zip(s.find(")")) {
        return Some(&s[a + 1..b]);
    }
    None
}

pub fn extract_value_and_parse<T: FromStr>(s: &str) -> Option<T> {
    if let Ok(t) = FromStr::from_str(s) {
        return Some(t);
    } else {
        for (a, b) in s.find("(").zip(s.find(")")) {
            return FromStr::from_str(&s[(a + 1)..b]).ok();
        }
    };
    None
}

pub fn extract_datetime_string(s: &str) -> Option<&str> {
    for a in s.find("(") {
        return Some(&s[..a]);
    }
    None
}

pub fn extract_datetime<'a>(s: &'a str) -> Option<NaiveDateTime> {
    for a in s.find("(") {
        return NaiveDateTime::parse_from_str(&s[..a], "%Y-%m-%dT%H:%M:%S%.9f").ok();
    }
    None
}

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
