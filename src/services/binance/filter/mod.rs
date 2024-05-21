pub mod error;
pub mod market;

type SymbolFilterResult<T> = Result<T, error::SymbolFilterError>;

mod current {
    use super::SymbolFilterResult;
    use crate::services::binance::filter::error::SymbolFilterError;
    use crate::services::binance::types::Quantity;

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
