#![allow(non_snake_case)]
use crate::from_record_batch::*;
use crate::{
    CombinationProduct, DeleteOrder, EquilibriumPrice, Executed, ExecutionWithPriceInfo, LegPrice,
    ProductInfo, PutOrder, SecondTag, SystemEventInfo, TickSize, TradingStatusInfo,
};
use chrono::NaiveDateTime;
use datafusion::arrow::error::ArrowError;
use datafusion::arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};

macro_rules! dclr_message_enum {
    ($($ident:ident,)*) => {
        #[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Hash, Ord)]
        #[serde(tag = "tag")]
        pub enum MessageEnum {
            $( $ident($ident), )*
        }

        impl MessageEnum {

            pub fn tag(&self) -> char {
                match self {
                    $( MessageEnum::$ident(_) => $ident::TAG, )*
                }
            }

            pub fn timestamp(&self) -> NaiveDateTime {
                match self {
                    $( MessageEnum::$ident(x) => x.timestamp, )*
                }
            }
        }

        impl FromRecordBatch for MessageEnum {
            fn validate(rb: &RecordBatch) -> bool {
                $(
                    let $ident = $ident::validate(rb);
                )*
                $(
                    $ident &&
                ) * true
            }
            fn from_record_batch(rb: &RecordBatch) -> Result<Vec<Self>, Vec<FromRecordBatchError>> {
                $(
                    let $ident = $ident::validate(rb);
                    if $ident {
                        return match $ident::from_record_batch(rb) {
                            Ok(i) => Ok(i.into_iter().map(|i| MessageEnum::$ident(i)).collect()),
                            Err(e) => Err(e)
                        }
                    }
                )*

                Err(vec![])
            }
        }

        $(
            impl TryFrom<MessageEnum> for $ident {
                type Error = &'static str;
                fn try_from(msg_enum: MessageEnum) -> Result<Self, Self::Error> {
                    match msg_enum {
                        MessageEnum::$ident(item) => Ok(item),
                        _ => Err(stringify!($ident))
                    }
                }
            }
        ) *

        impl TryFrom<String> for MessageEnum {
            type Error = String;
            fn try_from(string: String) -> Result<Self, Self::Error> {
                $(
                    if let Ok(i) = $ident::try_from(string.as_str()) {
                        return Ok(MessageEnum::$ident(i))
                    }
                ) *
                return Err(string)
            }
        }

    };
}

dclr_message_enum!(
    CombinationProduct,
    DeleteOrder,
    EquilibriumPrice,
    Executed,
    ExecutionWithPriceInfo,
    LegPrice,
    ProductInfo,
    PutOrder,
    SecondTag,
    SystemEventInfo,
    TickSize,
    TradingStatusInfo,
);

impl MessageEnum {
    /// create json from record batches then
    pub fn from_record_batches(batches: &[RecordBatch]) -> Result<Vec<MessageEnum>, ArrowError> {
        let list = datafusion::arrow::json::writer::record_batches_to_json_rows(batches)?
            .into_iter()
            .filter_map(|val| serde_json::from_value(serde_json::Value::Object(val)).ok())
            .collect();

        Ok(list)
    }

    pub fn struct_name_to_tag(struct_name: &str) -> Option<char> {
        match struct_name {
            "CombinationProduct" => 'M',
            "DeleteOrder" => 'D',
            "EquilibriumPrice" => 'Z',
            "Executed" => 'E',
            "ExecutionWithPriceInfo" => 'C',
            "LegPrice" => 'P',
            "ProductInfo" => 'R',
            "PutOrder" => 'A',
            "SecondTag" => 'T',
            "SystemEventInfo" => 'S',
            "TickSize" => 'L',
            "TradingStatusInfo" => 'O',
            _ => return None,
        }
        .into()
    }

    pub fn tag_to_struct_name(tag: char) -> Option<&'static str> {
        match tag {
            'M' => "CombinationProduct",
            'D' => "DeleteOrder",
            'Z' => "EquilibriumPrice",
            'E' => "Executed",
            'C' => "ExecutionWithPriceInfo",
            'P' => "LegPrice",
            'R' => "ProductInfo",
            'A' => "PutOrder",
            'T' => "SecondTag",
            'S' => "SystemEventInfo",
            'L' => "TickSize",
            'O' => "TradingStatusInfo",
            _ => return None,
        }
        .into()
    }
}
