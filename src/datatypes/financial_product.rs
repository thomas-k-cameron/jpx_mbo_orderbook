use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Hash, Ord, Clone, Copy)]
pub enum FinancialProduct {
    Option = 1,
    Future = 3,
    Combo = 11,
}

impl FromStr for FinancialProduct {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "1" => FinancialProduct::Option,
            "3" => FinancialProduct::Future,
            "11" => FinancialProduct::Combo,
            _ => return Err(()),
        })
    }
}
