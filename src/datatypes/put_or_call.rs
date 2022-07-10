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

impl TryFrom<i8> for PutOrCall {
    type Error = ();
    fn try_from(int: i8) -> Result<Self, Self::Error> {
        match int {
            1 => Ok(PutOrCall::Call),
            2 => Ok(PutOrCall::Put),
            0 => Ok(PutOrCall::Combo),
            _ => Err(()),
        }
    }
}
