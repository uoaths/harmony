pub mod base_quantity {
    use binance::types::{
        SymbolFilter, SymbolInfo, SymbolLotSizeFilter, SymbolMarketLotSizeFilter,
        SymbolNotionalFilter,
    };

    use crate::services::binance::filter::error::SymbolFilterError;
    use crate::services::binance::filter::SymbolFilterResult;
    use crate::services::binance::math;
    use crate::services::binance::types::{Price, Quantity};

    pub fn filter(
        norms: &SymbolInfo,
        price: &Price,
        quantity: &Quantity,
    ) -> SymbolFilterResult<()> {
        use crate::services::binance::filter::current::filter_precision;

        for filter in norms.filters.iter() {
            match filter {
                SymbolFilter::LotSize(v) => filter_lot_size(&quantity, v)?,
                SymbolFilter::Notional(v) => filter_notional(&price, &quantity, v)?,
                SymbolFilter::MarketLotSize(v) => filter_market_lot_size(&quantity, v)?,
                _ => continue,
            };
        }

        filter_precision(&quantity, norms.base_asset_precision.into())?;

        Ok(())
    }

    pub fn correct(
        norms: &SymbolInfo,
        _price: &Price,
        quantity: &Quantity,
    ) -> SymbolFilterResult<Quantity> {
        use crate::services::binance::filter::current::correct_precision;

        let mut correct_quantity = quantity.clone();

        // Correct Step
        for filter in norms.filters.iter() {
            if let SymbolFilter::LotSize(v) = filter {
                correct_quantity = correct_lot_size(&correct_quantity, v)?;
            }
            if let SymbolFilter::MarketLotSize(v) = filter {
                correct_quantity = correct_market_lot_size(&correct_quantity, v)?;
            }
        }

        // Correct Precision
        let precision = norms.base_asset_precision as u32;
        correct_quantity = correct_precision(&correct_quantity, precision);

        Ok(correct_quantity)
    }

    pub fn filter_lot_size<'a>(
        quantity: &'a Quantity,
        filter: &'a SymbolLotSizeFilter,
    ) -> SymbolFilterResult<&'a Quantity> {
        let step_size = &math::to_decimal(&filter.step_size)?;
        let max_base_quantity = &math::to_decimal(&filter.max_qty)?;
        let min_base_quantity = &math::to_decimal(&filter.min_qty)?;

        if quantity > max_base_quantity {
            return Err(SymbolFilterError::LotSize(format!(
                "base quantity {} exceeds the maximum quantity {}",
                quantity, max_base_quantity
            )));
        }

        if quantity < min_base_quantity {
            return Err(SymbolFilterError::LotSize(format!(
                "base quantity {} does not reach the minimum quantity {}",
                quantity, min_base_quantity
            )));
        }

        if !step_size.is_zero() {
            if quantity % step_size != math::ZERO {
                return Err(SymbolFilterError::LotSize(format!(
                    "the quantity {} is not a multiple of the required step size {}.",
                    quantity, step_size
                )));
            }
        }

        Ok(quantity)
    }

    pub fn filter_market_lot_size<'a>(
        quantity: &'a Quantity,
        filter: &'a SymbolMarketLotSizeFilter,
    ) -> SymbolFilterResult<&'a Quantity> {
        let max_base_quantity = &math::to_decimal(&filter.max_qty)?;
        let min_base_quantity = &math::to_decimal(&filter.min_qty)?;

        if quantity > max_base_quantity {
            return Err(SymbolFilterError::MarketLotSize(format!(
                "base quantity {} exceeds the maximum market quantity {}",
                quantity, max_base_quantity
            )));
        }

        if quantity < min_base_quantity {
            return Err(SymbolFilterError::MarketLotSize(format!(
                "base quantity {} does not reach the minimum market quantity {}",
                quantity, min_base_quantity
            )));
        }

        Ok(quantity)
    }

    pub fn filter_notional<'a>(
        price: &'a Price,
        quantity: &'a Quantity,
        filter: &SymbolNotionalFilter,
    ) -> SymbolFilterResult<&'a Quantity> {
        let notional = price * quantity;
        if filter.apply_max_to_market {
            let max_notional = math::to_decimal(&filter.max_notional)?;
            if notional > max_notional {
                return Err(SymbolFilterError::Notional(format!(
                    "the notional value of {} exceeds the maximum allowed notional value of {} for the market",
                    notional, max_notional
                )));
            }
        }

        if filter.apply_min_to_market {
            let min_notional = math::to_decimal(&filter.min_notional)?;
            if notional < min_notional {
                return Err(SymbolFilterError::Notional(format!(
                    "the notional value of {} * {} = {} does not meet the minimum required notional value of {} for the market",
                    price, quantity, notional, min_notional
                )));
            }
        }

        Ok(quantity)
    }

    pub fn correct_lot_size(
        quantity: &Quantity,
        filter: &SymbolLotSizeFilter,
    ) -> SymbolFilterResult<Quantity> {
        let step_size = math::to_decimal(&filter.step_size)?;

        if step_size.is_zero() {
            return Ok(quantity.clone());
        }

        Ok(quantity - (quantity % step_size))
    }

    pub fn correct_market_lot_size(
        quantity: &Quantity,
        filter: &SymbolMarketLotSizeFilter,
    ) -> SymbolFilterResult<Quantity> {
        let step_size = math::to_decimal(&filter.step_size)?;

        if step_size.is_zero() {
            return Ok(quantity.clone());
        }

        Ok(quantity - (quantity % step_size))
    }
}

pub mod quote_quantity {
    use binance::types::{SymbolFilter, SymbolInfo, SymbolMinNotionalFilter, SymbolNotionalFilter};

    use crate::services::binance::filter::error::SymbolFilterError;
    use crate::services::binance::filter::SymbolFilterResult;
    use crate::services::binance::math;
    use crate::services::binance::types::{Price, Quantity};

    pub fn filter(
        norms: &SymbolInfo,
        _price: &Price,
        quantity: &Quantity,
    ) -> SymbolFilterResult<()> {
        use crate::services::binance::filter::current::filter_precision;

        // precision
        let precision = norms.quote_asset_precision as u32;
        let quantity = filter_precision(quantity, precision)?;

        // filter
        for filter in norms.filters.iter() {
            match filter {
                SymbolFilter::Notional(v) => filter_notional(quantity, v)?,
                SymbolFilter::MinNotional(v) => filter_min_notional(quantity, v)?,
                _ => continue,
            };
        }

        Ok(())
    }

    pub fn correct(
        norms: &SymbolInfo,
        _price: &Price,
        quantity: &Quantity,
    ) -> SymbolFilterResult<Quantity> {
        use crate::services::binance::filter::current::correct_precision;

        let mut correct_quantity = quantity.clone();

        let precision = norms.quote_asset_precision as u32;
        correct_quantity = correct_precision(quantity, precision);

        Ok(correct_quantity)
    }

    pub fn filter_min_notional<'a>(
        quantity: &'a Quantity,
        filter: &'a SymbolMinNotionalFilter,
    ) -> SymbolFilterResult<&'a Quantity> {
        if !filter.apply_to_market {
            return Ok(quantity);
        }

        let min_notional = &math::to_decimal(&filter.min_notional)?;
        if quantity < min_notional {
            return Err(SymbolFilterError::MinNotional(format!(
                "the notional value of {} does not meet the minimum required notional value of {} for the market",
                quantity, min_notional
            )));
        }

        Ok(quantity)
    }

    pub fn filter_notional<'a>(
        quantity: &'a Quantity,
        filter: &'a SymbolNotionalFilter,
    ) -> SymbolFilterResult<&'a Quantity> {
        if filter.apply_max_to_market {
            let max_notional = &math::to_decimal(&filter.max_notional)?;
            if quantity > max_notional {
                return Err(SymbolFilterError::Notional(format!(
                    "the notional value of {} exceeds the maximum allowed notional value of {} for the market",
                    quantity, max_notional
                )));
            }
        }

        if filter.apply_min_to_market {
            let min_notional = &math::to_decimal(&filter.min_notional)?;
            if quantity < min_notional {
                return Err(SymbolFilterError::Notional(format!(
                    "the notional value of {} does not meet the minimum required notional value of {} for the market",
                    quantity, min_notional
                )));
            }
        }

        Ok(quantity)
    }
}

#[cfg(test)]
mod tests {
    use binance::types::SymbolInfo;
    use rust_decimal::Decimal;

    const SYMBOL_PRICE: &str = "3685.96000000";
    const SYMBOL_NORMS: &str = r#"{"allowTrailingStop":true,"allowedSelfTradePreventionModes":["EXPIRE_TAKER","EXPIRE_MAKER","EXPIRE_BOTH"],"baseAsset":"ETH","baseAssetPrecision":8,"baseCommissionPrecision":8,"cancelReplaceAllowed":true,"defaultSelfTradePreventionMode":"EXPIRE_MAKER","filters":[{"filterType":"PRICE_FILTER","maxPrice":"1000000.00000000","minPrice":"0.01000000","tickSize":"0.01000000"},{"filterType":"LOT_SIZE","maxQty":"9000.00000000","minQty":"0.00010000","stepSize":"0.00010000"},{"filterType":"ICEBERG_PARTS","limit":10},{"filterType":"MARKET_LOT_SIZE","maxQty":"1701.08445000","minQty":"0.00000000","stepSize":"0.00000000"},{"filterType":"TRAILING_DELTA","maxTrailingAboveDelta":2000,"maxTrailingBelowDelta":2000,"minTrailingAboveDelta":10,"minTrailingBelowDelta":10},{"askMultiplierDown":"0.2","askMultiplierUp":"5","avgPriceMins":5,"bidMultiplierDown":"0.2","bidMultiplierUp":"5","filterType":"PERCENT_PRICE_BY_SIDE"},{"applyMaxToMarket":false,"applyMinToMarket":true,"avgPriceMins":5,"filterType":"NOTIONAL","maxNotional":"9000000.00000000","minNotional":"5.00000000"},{"filterType":"MAX_NUM_ORDERS","maxNumOrders":200},{"filterType":"MAX_NUM_ALGO_ORDERS","maxNumAlgoOrders":5}],"icebergAllowed":true,"isMarginTradingAllowed":true,"isSpotTradingAllowed":true,"ocoAllowed":true,"orderTypes":["LIMIT","LIMIT_MAKER","MARKET","STOP_LOSS_LIMIT","TAKE_PROFIT_LIMIT"],"otoAllowed":false,"permissionSets":[["SPOT","MARGIN","TRD_GRP_004","TRD_GRP_005","TRD_GRP_006","TRD_GRP_009","TRD_GRP_010","TRD_GRP_011","TRD_GRP_012","TRD_GRP_013","TRD_GRP_014","TRD_GRP_015","TRD_GRP_016","TRD_GRP_017","TRD_GRP_018","TRD_GRP_019","TRD_GRP_020","TRD_GRP_021","TRD_GRP_022","TRD_GRP_023","TRD_GRP_024","TRD_GRP_025"]],"permissions":[],"quoteAsset":"USDT","quoteAssetPrecision":8,"quoteCommissionPrecision":8,"quoteOrderQtyMarketAllowed":true,"quotePrecision":8,"status":"TRADING","symbol":"ETHUSDT"}"#;

    fn symbol_norms() -> SymbolInfo {
        serde_json::from_str(SYMBOL_NORMS).unwrap()
    }

    fn symbol_price() -> Decimal {
        decimal(&SYMBOL_PRICE.to_string())
    }

    pub fn decimal(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }

    #[cfg(test)]
    mod tests_base_quantity {
        use binance::types::SymbolFilter;

        use crate::services::binance::filter::spot_market::base_quantity::*;

        use super::{decimal, symbol_norms, symbol_price};

        #[test]
        pub fn test_filter_lot_size() {
            for i in symbol_norms().filters.iter() {
                if let SymbolFilter::LotSize(filter) = i {
                    let quantity = &decimal("0.0001");
                    let correct = filter_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("0.0001000");
                    let correct = filter_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("7.50100000");
                    let correct = filter_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("0.0001500");
                    let correct = filter_lot_size(quantity, filter);
                    assert!(correct.is_err());

                    let quantity = &decimal("0.0001500");
                    let correct = filter_lot_size(quantity, filter);
                    assert!(correct.is_err());

                    let quantity = &decimal("9001.0001500");
                    let correct = filter_lot_size(quantity, filter);
                    assert!(correct.is_err());

                    break;
                }
            }
        }

        #[test]
        fn test_filter_market_lot_size() {
            for i in symbol_norms().filters.iter() {
                if let SymbolFilter::MarketLotSize(filter) = i {
                    let quantity = &decimal("0.00001");
                    let correct = filter_market_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("0.00001000");
                    let correct = filter_market_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("7.510100000");
                    let correct = filter_market_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("0.00001500");
                    let correct = filter_market_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("1800.00000");
                    let correct = filter_market_lot_size(quantity, filter);
                    assert!(correct.is_err());

                    let quantity = &decimal("9001.00001500");
                    let correct = filter_market_lot_size(quantity, filter);
                    assert!(correct.is_err());

                    break;
                }
            }
        }

        #[test]
        fn test_filter_notional() {
            let price = &symbol_price();

            for i in symbol_norms().filters.iter() {
                if let SymbolFilter::Notional(filter) = i {
                    let quantity = &decimal("0.0015");
                    let correct = filter_notional(price, quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("0.0005");
                    let correct = filter_notional(price, quantity, filter);
                    assert!(correct.is_err());

                    let quantity = &decimal("0.00000001");
                    let correct = filter_notional(price, quantity, filter);
                    assert!(correct.is_err());

                    break;
                }
            }
        }

        #[test]
        fn test_correct_lot_size() {
            for i in symbol_norms().filters.iter() {
                if let SymbolFilter::LotSize(filter) = i {
                    let quantity = &decimal("0.00001");
                    let correct = &correct_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, quantity);

                    let quantity = &decimal("0.00001500");
                    let correct = &correct_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, &decimal("0.00001000"));

                    let quantity = &decimal("0.000000");
                    let correct = &correct_lot_size(quantity, filter).unwrap();
                    assert_eq!(correct, &decimal("0.000000"));
                }

                break;
            }
        }
    }
}
