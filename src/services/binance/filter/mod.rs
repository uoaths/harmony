pub mod error;
pub mod spot;

type SymbolFilterResult<T> = Result<T, error::SymbolFilterError>;

mod current {
    use ploy::types::Quantity;

    use super::SymbolFilterResult;
    use crate::services::binance::filter::error::SymbolFilterError;

    pub fn filter_precision<'a>(
        quantity: &'a Quantity,
        precision: u32,
    ) -> SymbolFilterResult<&Quantity> {
        let scale = quantity.scale();

        if scale > precision {
            return Err(SymbolFilterError::Precision(format!(
                "the quantity {} exceeds the maximum allowed precision of {}. Current precision is {}.",
                quantity, precision, scale
            )));
        }

        Ok(&quantity)
    }

    pub fn correct_precision(quantity: &Quantity, precision: u32) -> Quantity {
        quantity.trunc_with_scale(precision)
    }
}

fn dec(value: &str) -> Result<ploy::types::Decimal, ploy::error::Error> {
    use std::str::FromStr;
    ploy::types::Decimal::from_str(value)
}
