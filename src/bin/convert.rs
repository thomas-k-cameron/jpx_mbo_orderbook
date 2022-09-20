use std::{fs::read_dir, thread::spawn};

use chrono::NaiveDateTime;
use jpx_mbo_orderbook::{from_filepath, MessageEnum};
use serde::{Serialize, Deserialize};
use tokio::{fs, io};

fn main() {
tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(_main());
}

async fn _main() -> io::Result<()> {
    let mut iter = fs::read_dir("./data").await.unwrap();
    while let Some(i) = iter.next_entry().await? {
        let item = from_filepath(i.path()).await;
        println!("{:#?}", item.unknown);
        for (key, val) in item.itch {
            let row = MessageJsonRow {
                key, val
            };
            
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct MessageJsonRow {
    key: NaiveDateTime,
    val: Vec<MessageEnum>
}