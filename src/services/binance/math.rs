use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Range(Decimal, Decimal);

impl Range {
    fn min(&self) -> &Decimal {
        if self.0 < self.1 {
            return &self.0;
        }

        &self.1
    }

    fn max(&self) -> &Decimal {
        if self.0 < self.1 {
            return &self.1;
        }

        &self.0
    }

    fn is_within(&self, value: &Decimal) -> bool {
        self.min() < value && value < self.max()
    }
}

pub fn is_within_price_ranges(price: &Price, ranges: &Vec<Range>) -> bool {
    if ranges.is_empty() {
        return false;
    }

    for range in ranges.iter() {
        if range.is_within(price) {
            return true;
        }

        continue;
    }

    false
}

use std::str::FromStr;

use rust_decimal::{Decimal, Error};

use super::types::Price;

pub fn to_decimal(value: &String) -> Result<Decimal, Error> {
    Decimal::from_str(value)
}

pub const ZERO: Decimal = Decimal::ZERO;
