// automatically generated
use crate::{tag_guard, Side};

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::util::{extract_datetime, extract_value};

///
///6.4.1 新規注文タグ （タグ ID ： A ）
///
///（１） タグ内容
///
///新規注文に係る気配情報を提供する。
///
///（２） タグ出力タイミング
///
///新規注文の発注のタイミングで出力
///
///注文訂正時は、削除注文タグによる取消し後、訂正後の注文に係る新規注文タグが出力される。
///
///朝、銘柄情報基本タグが配信されたのちに、リロードされた GTD/GTC 注文に関わる新規注文タグが出力される。
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Hash, Ord, Clone, Copy)]
pub struct PutOrder {
    pub timestamp: NaiveDateTime,
    pub channel: char,
    pub date: i64,
    pub order_book_id: u64,
    pub order_book_position: i64,
    pub order_id: u64,
    pub price: i64,
    pub quantity: i64,
    pub side: Side,
}

impl_message! {
    name: PutOrder 'A';
    pub timestamp: NaiveDateTime,
    pub channel: char,
    pub date: i64,
    pub order_book_id: u64,
    pub order_book_position: i64,
    pub order_id: u64,
    pub price: i64,
    pub quantity: i64,
    pub side: Side,
}

impl TryFrom<&str> for PutOrder {
    type Error = ();
    //(s: &str, row_no: i64, filename: i64)
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        tag_guard!('A', s);
        let mut iter = s.split(",").skip(1);
        let timestamp = extract_datetime(iter.next().ok_or(())?).ok_or(())?;
        let order_id = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let order_book_id = FromStr::from_str(extract_value(iter.next().ok_or(())?).ok_or(())?)
            .ok()
            .ok_or(())?;
        let side = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let order_book_position = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let quantity = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let price = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        Ok(Self {
            channel: char::MAX,
            date: i64::MIN,
            timestamp,
            order_book_id,
            order_book_position,
            order_id,
            price,
            quantity,
            side,
        })
    }
}
