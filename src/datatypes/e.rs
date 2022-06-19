// automatically generated
use crate::{tag_guard, Side};

use serde::{Deserialize, Serialize};

use std::str::FromStr;

///
///6.4.2 約定通知タグ （タグ ID ： E ）
///
///（１） タグ内容
///
///約定通知を提供する。
///
///既発注の注文サイドの注文受付番号をセットするので（売り注文が存在し、買い注文により約定した場合は、売り注文に係る注文受付番号を
///
///セット）、それにより注文情報を把握する。
///
///（２） タグ出力タイミング
///
///注文の部分又は全約定の都度出力する。
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub struct Executed {
    pub timestamp: NaiveDateTime,
    pub channel: char,
    pub date: i64,
    pub combo_group_id: i64,
    pub executed_quantity: i64,
    pub match_id: String,
    pub order_book_id: u64,
    pub order_id: u64,
    pub side: Side,
}

impl_message! {
    name: Executed 'E';
    pub timestamp: NaiveDateTime,
    pub channel: char,
    pub date: i64,
    pub combo_group_id: i64,
    pub executed_quantity: i64,
    pub match_id: String,
    pub order_book_id: u64,
    pub order_id: u64,
    pub side: Side,
}

impl TryFrom<&str> for Executed {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        tag_guard!('E', s);
        let mut iter = s.split(",").skip(1);
        let timestamp = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let combo_group_id = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let executed_quantity = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let match_id = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let order_book_id = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let order_id = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let side = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        Ok(Self {
            channel: char::MAX,
            date: i64::MIN,
            timestamp,
            combo_group_id,
            executed_quantity,
            match_id,
            order_book_id,
            order_id,
            side,
        })
    }
}
