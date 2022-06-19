use crate::Side;
use chrono::NaiveDateTime;
use datafusion::arrow::datatypes::{DataType, Field, TimeUnit};
use std::str;

use crate::{FinancialProduct, PutOrCall};
#[macro_export]
macro_rules! impl_message {
    (
        name: $name:ident $char:literal;
        pub timestamp: $_:ty,
        $ ( pub $field:ident: $dt:ty, ) *
    ) => {
        use chrono::NaiveDateTime;
        use std::collections::BTreeMap;
        use datafusion::arrow::{datatypes::{Schema}, record_batch::RecordBatch};
        use crate::{into_field, IntoRecordBatch, IntoArrayRef};
        impl TryFrom<&BTreeMap<String, String>> for $name {
            type Error = (&'static str, Option<String>);
            fn try_from(value: &BTreeMap<String, String>) -> Result<Self, Self::Error> {
                    if value.get("tag") != Some(&$char.to_string()) {
                        return Err(("TAG_CHAR_ERROR", None));
                    }
                     Ok(Self {
                        timestamp: {
                            let key = "timestamp";
                            let item = value.get(key);
                            if let Some(i) = item {
                                if let Some(i) = crate::util::extract_datetime(i) {
                                    i
                                }else {
                                    NaiveDateTime::from_timestamp(i.parse::<i64>().unwrap(), 0)
                                }
                            } else {
                                return Err((key, None))
                            }
                        },
                        $(
                            $field: {
                                let key = stringify!($field);
                                let item = value.get(key);
                                if let Some(s) = item {
                                    if let Some(r) = crate::util::extract_value_and_parse(s) {
                                        r
                                    } else {
                                        return Err((key, Some(s.to_string())))
                                    }
                                } else {
                                    return Err((key, None))
                                }
                            },
                        ) *
                    }
                )
            }
        }

        impl From<&$name> for Schema {
            fn from(_: &$name) -> Self {
                Schema::new(
                    vec![
                        into_field::<NaiveDateTime>("timestamp"),
                        $(
                            into_field::<$dt>(stringify!($field)),
                        )*
                    ]
                )
            }
        }

        impl IntoRecordBatch for $name {
            fn into_record_batch(stack: Vec<Self>) -> RecordBatch {
                let schema = Schema::new(
                    vec![
                        into_field::<NaiveDateTime>("timestamp"),
                        $(
                            into_field::<$dt>(stringify!($field)),
                        )*
                    ]
                );
                let columns = vec![
                    IntoArrayRef::into_array_ref(
                        stack.iter().map(|i| i.timestamp.clone()).collect::<Vec<_>>()
                    ),
                    $(
                        IntoArrayRef::into_array_ref(
                            stack.iter().map(|i| i.$field.clone()).collect::<Vec<_>>()
                        ),
                    ) *
                ];
                let schema_ptr = std::sync::Arc::new(schema);
                RecordBatch::try_new(schema_ptr.clone(), columns).expect(format!("{:?}", stringify!($name)).as_str())
            }
        }
    };
}

pub trait IntoField {
    fn field(s: &str) -> Field;
    fn datatype() -> DataType;
}
pub fn into_field<T: IntoField>(s: &str) -> Field {
    T::field(s)
}

macro_rules! impl_into_field {
    ($( $ty:ty, $dt:expr ) *) => {
        $(
            impl IntoField for $ty {
                fn field(s: &str) -> Field {
                    Field::new(s, $dt, false)
                }
                fn datatype() -> DataType {
                    $dt
                }
            }
        ) *
    };
}

impl_into_field!(
    NaiveDateTime, DataType::Timestamp(TimeUnit::Nanosecond, None)
    u64, DataType::UInt64
    i64, DataType::Int64
    String, DataType::Utf8
    Option<String>, DataType::Utf8
    Side, DataType::Boolean
    PutOrCall, DataType::Int8
    FinancialProduct, DataType::Int8
    char, DataType::UInt32
);
