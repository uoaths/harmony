pub mod error {
    #[derive(Debug)]
    pub enum SymbolFilterError {
        Decimal(String),
        LotSize(String),
        Notional(String),
        MinNotional(String),
        MarketLotSize(String),
    }

    impl std::error::Error for SymbolFilterError {}
    impl std::fmt::Display for SymbolFilterError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let message = match self {
                Self::Decimal(e) => e.to_string(),
                Self::LotSize(e) => format!("LOTSIZE {}", e),
                Self::Notional(e) => format!("NOTIONAL {}", e),
                Self::MinNotional(e) => format!("MIN_NOTIONAL {}", e),
                Self::MarketLotSize(e) => format!("MARKET_LOT_SIZE {}", e),
            };

            write!(f, "FILTER {}", message)
        }
    }

    impl From<rust_decimal::Error> for SymbolFilterError {
        fn from(value: rust_decimal::Error) -> Self {
            Self::Decimal(value.to_string())
        }
    }
}

pub mod spot_market {
    use crate::services::binance::math;
    use crate::services::binance::types;

    use super::error::SymbolFilterError;

    pub mod base_quantity {
        use binance::types::{SymbolFilter, SymbolInfo};

        use super::types::{Decimal, Quantity};
        use super::{math, SymbolFilterError};

        pub fn filter(
            norms: &SymbolInfo,
            price: &Decimal,
            base_quantity: &Decimal,
        ) -> Result<Quantity, SymbolFilterError> {
            // precision
            let correct_quantity = base_quantity.trunc_with_scale(norms.base_asset_precision.into());

            // filter
            let correct_quantity = step_correct_base_quantity(&correct_quantity, &norms.filters)?;
            for filter in norms.filters.iter() {
                match filter {
                    SymbolFilter::LotSize(v) => {
                        let max_base_quantity = math::to_decimal(&v.max_qty)?;
                        let min_base_quantity = math::to_decimal(&v.min_qty)?;

                        if correct_quantity > max_base_quantity {
                            return Err(SymbolFilterError::LotSize(format!(
                                "base quantity {} exceeds the maximum quantity {}",
                                correct_quantity, max_base_quantity
                            )));
                        }

                        if correct_quantity < min_base_quantity {
                            return Err(SymbolFilterError::LotSize(format!(
                                "base quantity {} does not reach the minimum quantity {}",
                                correct_quantity, min_base_quantity
                            )));
                        }
                    }

                    SymbolFilter::MarketLotSize(v) => {
                        let max_base_quantity = math::to_decimal(&v.max_qty)?;
                        let min_base_quantity = math::to_decimal(&v.min_qty)?;

                        if correct_quantity > max_base_quantity {
                            return Err(SymbolFilterError::MarketLotSize(format!(
                                "base quantity {} exceeds the maximum market quantity {}",
                                correct_quantity, max_base_quantity
                            )));
                        }

                        if correct_quantity < min_base_quantity {
                            return Err(SymbolFilterError::MarketLotSize(format!(
                                "base quantity {} does not reach the minimum market quantity {}",
                                correct_quantity, min_base_quantity
                            )));
                        }
                    }

                    SymbolFilter::Notional(v) => {
                        let notional = price * correct_quantity;
                        if v.apply_max_to_market {
                            let max_notional = math::to_decimal(&v.max_notional)?;
                            if notional > max_notional {
                                return Err(SymbolFilterError::Notional(format!(
                                    "the notional value of {} exceeds the maximum allowed notional value of {} for the market",
                                    notional, max_notional
                                )));
                            }
                        }

                        if v.apply_min_to_market {
                            let min_notional = math::to_decimal(&v.min_notional)?;
                            if notional < min_notional {
                                return Err(SymbolFilterError::Notional(format!(
                                    "the notional value of {} does not meet the minimum required notional value of {} for the market",
                                    notional, min_notional
                                )));
                            }
                        }
                    }

                    _ => continue,
                }
            }

            Ok(correct_quantity)
        }

        fn step_correct_base_quantity(
            base_quantity: &Quantity,
            filters: &Vec<SymbolFilter>,
        ) -> Result<Quantity, SymbolFilterError> {
            let mut correct_base_quantity = base_quantity.clone();

            for filter in filters.iter() {
                if let SymbolFilter::LotSize(v) = filter {
                    let step_size = math::to_decimal(&v.step_size)?;
                    if step_size.is_zero() {
                        continue;
                    }

                    correct_base_quantity =
                        correct_base_quantity - (correct_base_quantity % step_size);
                }
                if let SymbolFilter::MarketLotSize(v) = filter {
                    let step_size = math::to_decimal(&v.step_size)?;
                    if step_size.is_zero() {
                        continue;
                    }

                    correct_base_quantity =
                        correct_base_quantity - (correct_base_quantity % step_size);
                }
            }

            Ok(correct_base_quantity)
        }
    }

    pub mod quote_quantity {
        use binance::types::{SymbolFilter, SymbolInfo};

        use super::types::{Decimal, Quantity};
        use super::{math, SymbolFilterError};

        pub fn filter(
            norms: &SymbolInfo,
            _price: &Decimal,
            quote_quantity: &Decimal,
        ) -> Result<Quantity, SymbolFilterError> {
            // precision
            let correct_quote_quantity = quote_quantity.trunc_with_scale(norms.quote_asset_precision.into());

            // filter
            for filter in norms.filters.iter() {
                match filter {
                    SymbolFilter::MinNotional(v) => {
                        if v.apply_to_market {
                            let min_notional = math::to_decimal(&v.min_notional)?;
                            if correct_quote_quantity < min_notional {
                                return Err(SymbolFilterError::MinNotional(format!(
                                    "the notional value of {} does not meet the minimum required notional value of {} for the market",
                                    correct_quote_quantity, min_notional
                                )));
                            }
                        }
                    }

                    SymbolFilter::Notional(v) => {
                        if v.apply_max_to_market {
                            let max_notional = math::to_decimal(&v.max_notional)?;
                            if correct_quote_quantity > max_notional {
                                return Err(SymbolFilterError::Notional(format!(
                                    "the notional value of {} exceeds the maximum allowed notional value of {} for the market",
                                    correct_quote_quantity, max_notional
                                )));
                            }
                        }

                        if v.apply_min_to_market {
                            let min_notional = math::to_decimal(&v.min_notional)?;
                            if correct_quote_quantity < min_notional {
                                return Err(SymbolFilterError::Notional(format!(
                                    "the notional value of {} does not meet the minimum required notional value of {} for the market",
                                    correct_quote_quantity, min_notional
                                )));
                            }
                        }
                    }

                    _ => continue,
                }
            }

            Ok(correct_quote_quantity)
        }
    }
}
