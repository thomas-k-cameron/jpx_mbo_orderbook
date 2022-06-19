use datafusion::arrow::record_batch::RecordBatch;

pub trait IntoRecordBatch: Sized {
    fn into_record_batch(iter: Vec<Self>) -> RecordBatch;
}
