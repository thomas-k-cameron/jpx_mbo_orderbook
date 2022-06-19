// automatically generated

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::tag_guard;

///
///6.3.6 取引ステータス情報タグ （タグ ID ： O ）
///
///（１） タグ内容
///
///取引銘柄の取引ステータス情報を提供する。
///
///（２） タグ出力タイミング
///
///取引ステータスの更新のタイミングで、銘柄毎に提供する。
///
///連続 DCB となるケースでは、DCB から DCB へ遷移する時に、一旦 ZARABA のステータスを配信し、同時に DCB のステータスを配信する
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub struct TradingStatusInfo {
    pub timestamp: NaiveDateTime,
    pub channel: char,
    pub date: i64,
    pub order_book_id: u64,
    pub state_name: String,
}

impl_message! {
    name: TradingStatusInfo 'O';
    pub timestamp: NaiveDateTime,
    pub channel: char,
    pub date: i64,
    pub order_book_id: u64,
    pub state_name: String,
}

impl TryFrom<&str> for TradingStatusInfo {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        tag_guard!('O', s);
        let mut iter = s.split(",").skip(1);
        let timestamp = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let order_book_id = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        let state_name = FromStr::from_str(iter.next().ok_or(())?).ok().ok_or(())?;
        Ok(Self {
            channel: char::MAX,
            date: i64::MIN,
            timestamp,
            order_book_id,
            state_name,
        })
    }
}
