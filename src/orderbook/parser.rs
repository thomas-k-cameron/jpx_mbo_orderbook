use std::{
    collections::{HashMap},
    path::Path,
};

use chrono::NaiveDateTime;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};
use crate::MessageEnum;
use crate::{datatypes::*};



pub fn from_raw_file(file: String) -> JPXMBOParseResult {
    let mut parser = JPXMBOParser::default();
    for row in file.split("\n").map(|i| i.to_string()) {
        parser.parse_line(row);
    }

    parser.complete_parsing()
}

pub async fn from_filepath(filepath: impl AsRef<Path>) -> JPXMBOParseResult {
    let mut parser = JPXMBOParser::default();
    let mut lines = {
        let file = File::open(filepath).await.unwrap();
        BufReader::new(file).lines()
    };
    
    loop {
        match lines.next_line().await {
            Ok(Some(line)) => {
                parser.parse_line(line);
            }
            Err(e) => {
                println!("{e}");
                break
            },
            _ => break,
        };
    }
    parser.complete_parsing()
}


#[derive(Default, PartialEq, Eq)]
pub struct JPXMBOParseResult {
    pub itch: Vec<(NaiveDateTime, Vec<MessageEnum>)>,
    pub unknown: Vec<String>,
}

#[derive(Default)]
pub struct JPXMBOParser {
    temp: Vec<MessageEnum>,
    last_timestamp: NaiveDateTime,
    map: HashMap<NaiveDateTime, usize>,
    itch: Vec<(NaiveDateTime, Vec<MessageEnum>)>,
    unknown: Vec<String>
}

impl JPXMBOParser {
    pub async fn from_filepath(filepath: impl AsRef<Path>) -> JPXMBOParser {
        let mut parser = JPXMBOParser::default();
        let mut lines = {
            let file = File::open(filepath).await.unwrap();
            BufReader::new(file).lines()
        };
    
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    parser.parse_line(line);
                }
                Err(e) => {
                    println!("{e}");
                    break
                },
                _ => break,
            };
        }
        parser
    }
    pub fn from_string(file: String) -> JPXMBOParser {
        let mut parser = JPXMBOParser::default();
        for row in file.split("\n").map(|i| i.to_string()) {
            parser.parse_line(row);
        }
    
        parser
    }
    fn insert_temp(&mut self, timestamp: Option<NaiveDateTime>) {
        let timestamp = timestamp.unwrap_or(self.last_timestamp);
        match self.map.get(&timestamp) {
            Some(index) => {
                if let Some((_, val)) = self.itch.get_mut(*index) {
                    val.append(&mut self.temp);
                } else {
                    unreachable!();
                };
            }
            None => {
                let mut vector = Vec::with_capacity(self.temp.len());
                vector.append(&mut self.temp);
                self.itch.push((timestamp, vector));
                self.map.insert(timestamp, self.itch.len()-1);
            }
        };
    }
    pub fn parse_line(&mut self, s: String) {
        match MessageEnum::try_from(s) {
            Ok(i) => {
                let check = if let Some(temp_timestamp) = self.temp.first() {
                    i.timestamp() == temp_timestamp.timestamp()
                } else {
                    true
                };

                if !check {
                    self.insert_temp(i.timestamp().into());
                    self.last_timestamp = i.timestamp();
                };
                self.temp.push(i);
            }
            Err(e) => self.unknown.push(e),
        }
    }
    pub fn complete_parsing(mut self) -> JPXMBOParseResult {
        self.insert_temp(None);
        self.itch.sort_by(|(a, _), (b, _)| a.cmp(b));
        JPXMBOParseResult {
            itch: self.itch,
            unknown: self.unknown,
        }
    }
}
