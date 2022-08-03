use super::runtime::from_raw_file;
use serde_json;
use std::{fs, iter::repeat};

#[tokio::test]
pub async fn handle() {
    let filepath = "./data/srnd-itch_20210301_A.csv";
    let file = std::fs::read_to_string(filepath).unwrap();
    let stuff = from_raw_file(file);
    for i in stuff.unknown {
        println!("{:?}", i);
    }
}
