#[macro_export]
macro_rules! impl_message {
    (
        name: $name:ident $char:literal;
        pub timestamp: $_:ty,
        $ ( pub $field:ident: $dt:ty, ) *
    ) => {
        use chrono::NaiveDateTime;

        impl_message!(set_tag @ $name, $char);
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
            /// just validation. no detailed error reporting
            fn validate(rb: &RecordBatch) -> bool {
                $(
                    let mut $field = usize::MAX;
                ) *
                for (idx, i) in rb.schema().fields().iter().enumerate() {
                    match i.name().as_str() {
                        $(
                            stringify!($field) => $field = idx,
                        ) *
                        _ => ()
                    };
                }

                $(
                    if $field != usize::MAX {
                        return false
                    }
                ) *
                true
            }
            fn from_record_batch(rb: &RecordBatch) -> Result<Vec<$name>, Vec<FromRecordBatchError>> {
                let mut err_stack = vec![];
                $( let mut $field: Option<&<$dt as IntoField>::ArrayType> = None; )*
                let mut timestamp = None;
                // validate
                {
                    $(
                        let mut $field = usize::MAX;
                    ) *
                    for (idx, i) in rb.schema().fields().iter().enumerate() {
                        match i.name().as_str() {
                            $(
                                stringify!($field) => $field = idx,
                            ) *
                            _ => ()
                        };
                    }

                    $(
                        if $field != usize::MAX {
                            err_stack.push(
                                FromRecordBatchError {
                                    kind: FromRecordBatchErrorKind::ValidationError,
                                    name: stringify!($field)
                                }
                            )
                        }
                    ) *
                    if err_stack.len() > 0 {
                        return Err(err_stack)
                    }
                };
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
                                    name:  "timestamp"
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
                                        name: stringify!($field)
                                    };
                                    err_stack.push(err)
                                };
                            }
                        ) *
                        _ => err_stack.push(FromRecordBatchError{
                            kind: FromRecordBatchErrorKind::ColumnNotFound(i.name().to_string()),
                            name: ""
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
                                            name: stringify!($field),
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