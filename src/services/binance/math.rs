use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Range(pub f64, pub f64);

impl Range {
    pub fn min(&self) -> f64 {
        if self.0 < self.1 {
            return self.0;
        }

        self.1
    }

    pub fn max(&self) -> f64 {
        if self.0 < self.1 {
            return self.1;
        }

        self.0
    }

    pub fn is_within(&self, value: f64) -> bool {
        self.min() < value && value < self.max()
    }
}

use std::str::FromStr;

use rust_decimal::{Decimal, Error};

pub fn to_decimal(value: &String) -> Result<Decimal, Error> {
    Decimal::from_str(value)
}
