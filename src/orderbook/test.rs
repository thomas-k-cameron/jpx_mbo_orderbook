use std::fs;

use super::runtime::from_raw_file;

#[tokio::test]
async fn handle() {
    let filepath = "./data/srnd-itch_20210301_A.csv";
    let stuff = from_raw_file(filepath).await;
    assert!(stuff.is_ok())
}