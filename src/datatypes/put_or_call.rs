use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Hash, Ord, Clone, Copy)]
pub enum PutOrCall {
    Put = 1,
    Call = 2,
    Combo = 0,
}

impl FromStr for PutOrCall {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "1" | "CAL" => PutOrCall::Call,
            "2" | "PUT" => PutOrCall::Put,
            "0" => PutOrCall::Combo,
            _ => return Err(()),
        })
    }
}
