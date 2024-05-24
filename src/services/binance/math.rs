use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range<T>(T, T);

impl Range<Decimal> {
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

pub fn is_within_ranges(value: &Decimal, ranges: &Vec<Range<Decimal>>) -> bool {
    if ranges.is_empty() {
        return false;
    }

    for range in ranges.iter() {
        if range.is_within(value) {
            return true;
        }

        continue;
    }

    false
}

use std::str::FromStr;

use rust_decimal::{Decimal, Error};

pub fn to_decimal(value: &String) -> Result<Decimal, Error> {
    Decimal::from_str(value)
}

pub const ZERO: Decimal = Decimal::ZERO;
