use crate::Side;
use chrono::NaiveDateTime;
use datafusion::arrow::array::{
    ArrayRef, BooleanArray, Int64Array, Int8Array, StringBuilder, TimestampNanosecondArray,
    UInt32Array, UInt64Array,
};
use std::sync::Arc;

use crate::{FinancialProduct, PutOrCall};

pub trait IntoArrayRef: Sized {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef;
}

impl IntoArrayRef for NaiveDateTime {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let mut arr = TimestampNanosecondArray::builder(stack.len());
        arr.append_slice(
            &stack
                .iter()
                .map(|i| i.timestamp_nanos())
                .collect::<Vec<_>>()[..],
        )
        .unwrap();
        Arc::new(arr.finish())
    }
}

impl IntoArrayRef for char {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let mut arr = UInt32Array::builder(stack.len());
        arr.append_slice(&stack.iter().map(|i| *i as u32).collect::<Vec<u32>>()[..])
            .unwrap();
        Arc::new(arr.finish())
    }
}

impl IntoArrayRef for u64 {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let mut arr = UInt64Array::builder(stack.len());
        arr.append_slice(&stack[..]).unwrap();
        Arc::new(arr.finish())
    }
}

impl IntoArrayRef for i64 {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let mut arr = Int64Array::builder(stack.len());
        arr.append_slice(&stack[..]).unwrap();
        Arc::new(arr.finish())
    }
}

impl IntoArrayRef for String {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let mut builder = StringBuilder::new(stack.len());
        for i in stack {
            builder.append_value(i).unwrap();
        }

        Arc::new(builder.finish())
    }
}

impl IntoArrayRef for Option<String> {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let mut builder = StringBuilder::new(stack.len());

        for i in stack {
            builder.append_option(i).unwrap();
        }

        Arc::new(builder.finish())
    }
}

impl IntoArrayRef for Side {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let arr = BooleanArray::from_iter(stack.iter().map(|i| Some(i.is_buy())));
        Arc::new(arr)
    }
}

impl IntoArrayRef for PutOrCall {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let arr = Int8Array::from_iter(stack.iter().map(|i| Some((*i as isize) as i8)));
        Arc::new(arr)
    }
}

impl IntoArrayRef for FinancialProduct {
    fn into_array_ref(stack: Vec<Self>) -> ArrayRef {
        let arr = Int8Array::from_iter(stack.iter().map(|i| Some((*i as isize) as i8)));
        Arc::new(arr)
    }
}
