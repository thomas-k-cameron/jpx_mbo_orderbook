use crate::Side;
use chrono::NaiveDateTime;
use datafusion::arrow::datatypes::{DataType, Field, TimeUnit};
use datafusion::arrow::record_batch::RecordBatch;
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
        use datafusion::arrow::{array::Array, datatypes::{Schema}, record_batch::RecordBatch};
        use crate::{
            into_field,
            IntoRecordBatch,
            IntoArrayRef,
            from_btree::{FromRecordBatchError, FromRecordBatchErrorKind, FromRecordBatch, IntoField}
        };

        impl_message!(set_tag @ $name, $char);
        impl_message!(from_btree_map @ $char $name $ ( $field $dt ) *);
        impl_message!(schema @ $name $( $field $dt ) *);
        impl_message!(IntoRecordBatch @ $name $($field $dt) *);
        impl_message!(FromRecordBatch @ $name $($field $dt) *);
    };
    (set_tag @ $name:ident, $char:literal) => {
        impl $name {
            pub const TAG: char = $char;
        }
    };
    (from_btree_map @ $char:literal $name:ident $ ( $field:ident $dt:ty ) *) => {

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

    };
    (schema @ $name:ident $( $field:ident $dt:ty ) *) => {
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
    };
    (IntoRecordBatch @ $name:ident $( $field:ident $dt:ty ) *) => {
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
    (FromRecordBatch @ $name:ident $( $field:ident $dt:ty ) *) => {
        impl FromRecordBatch for $name {
            fn from_record_batch(rb: &RecordBatch) -> Result<Vec<$name>, Vec<FromRecordBatchError>> {
                let mut err_stack = vec![];
                $( let mut $field: Option<&<$dt as IntoField>::ArrayType> = None; )*
                let mut timestamp = None;
                // get record batch for each field
                for (idx, i) in rb.schema().fields().iter().enumerate() {
                    match i.name().as_str() {
                        "timestamp" => {
                            let arraytype = rb.column(idx)
                                    .as_any()
                                    .downcast_ref::<<NaiveDateTime as IntoField>::ArrayType>();
                            if let Some(column_array) = arraytype {
                                timestamp.replace(column_array);
                            } else {
                                let err = FromRecordBatchError{
                                    kind: FromRecordBatchErrorKind::Downcast,
                                    name: i.name().to_string()
                                };
                                err_stack.push(err)
                            };
                        }
                        $(
                            stringify!($field) => {
                                let arraytype = rb.column(idx)
                                    .as_any()
                                    .downcast_ref::<<$dt as IntoField>::ArrayType>();
                                if let Some(column_array) = arraytype {
                                    $field.replace(column_array);
                                } else {
                                    let err = FromRecordBatchError{
                                        kind: FromRecordBatchErrorKind::Downcast,
                                        name: i.name().to_string()
                                    };
                                    err_stack.push(err)
                                };
                            }
                        ) *
                        _ => err_stack.push(FromRecordBatchError{
                            kind: FromRecordBatchErrorKind::ColumnNotFound,
                            name: i.name().to_string()
                        })
                    };
                }

                if err_stack.len() > 0 {
                    Err(err_stack)
                } else {
                    // setup variables
                    let timestamp = timestamp.unwrap();
                    $(
                        let $field = $field.unwrap();
                    ) *

                    let mut stack = Vec::with_capacity(timestamp.len());

                    for i in 0..timestamp.len() {
                        let timestamp = NaiveDateTime::from_timestamp(timestamp.value(i), 0);
                        $(
                            let $field = impl_message!(FromRecordBatch struct_fields @ $field $field.value(i));
                        ) *
                        $(
                            let $field = {
                                if let Ok(i) = $field.try_into() {
                                    Some(i)
                                } else {
                                    err_stack.push(
                                        FromRecordBatchError {
                                            name: stringify!($field).to_string(),
                                            kind: FromRecordBatchErrorKind::TypeConversionFailed
                                        }
                                    );
                                    None
                                }
                            };
                        ) *

                        if err_stack.len() > 0 {
                            return Err(err_stack)
                        }
                        let item = $name {
                            timestamp,
                            $ ( $field: $field.unwrap(), ) * 
                        };
                        stack.push(item);
                    }
                    Ok(stack)
                }
            }
        }
    };

    (FromRecordBatch struct_fields @ channel $expr:expr) => {
        ($expr) as char
    };
    (FromRecordBatch struct_fields @ side $expr:expr) => {
        Side::from($expr)
    };
    (FromRecordBatch struct_fields @ $_:ident $expr:expr) => {
        $expr
    };
}

pub trait FromRecordBatch: Sized {
    fn from_record_batch(list: &RecordBatch) -> Result<Vec<Self>, Vec<FromRecordBatchError>>;
}
pub struct FromRecordBatchError {
    pub kind: FromRecordBatchErrorKind,
    pub name: String,
}
pub enum FromRecordBatchErrorKind {
    Downcast,
    ColumnNotFound,
    TypeConversionFailed
}

pub trait IntoField {
    type ArrayType;
    fn field(s: &str) -> Field;
    fn datatype() -> DataType;
}

pub fn into_field<T: IntoField>(s: &str) -> Field {
    T::field(s)
}

macro_rules! impl_into_field {
    ($( $ty:ty, $dt:expr, $array_dt:ident ) *) => {
        use datafusion::arrow::array::{
            TimestampNanosecondArray,
            UInt64Array,
            Int64Array,
            UInt8Array,
            StringArray,
            BooleanArray,
            Int8Array
        };
        $(
            impl IntoField for $ty {
                type ArrayType = $array_dt;
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
    NaiveDateTime, DataType::Timestamp(TimeUnit::Nanosecond, None), TimestampNanosecondArray
    u64, DataType::UInt64, UInt64Array
    i64, DataType::Int64, Int64Array
    String, DataType::Utf8, StringArray
    Option<String>, DataType::Utf8, StringArray
    Side, DataType::Boolean, BooleanArray
    PutOrCall, DataType::Int8, Int8Array
    FinancialProduct, DataType::Int8, Int8Array
    char, DataType::UInt8, UInt8Array
);
