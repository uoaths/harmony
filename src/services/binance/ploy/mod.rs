mod limit;

use serde::{Deserialize, Serialize};

use crate::services::binance::math::Range;
use crate::services::binance::types::{Price, Quantity};
use crate::time;

use super::types::{BaseQuantity, QuoteQuantity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub buying_prices: Vec<Range<Price>>,
    pub selling_prices: Vec<Range<Price>>,
    pub base_quantity: Quantity,
    pub quote_quantity: Quantity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub side: TradeSide,
    pub price: Price,
    pub base_quantity: Quantity,
    pub quote_quantity_commission: Quantity,
    pub timestamp: u128,
}

impl Trade {
    pub fn new(
        side: TradeSide,
        price: Price,
        base_quantity: Quantity,
        quote_quantity_commission: Quantity,
        timestamp: u128,
    ) -> Self {
        Self {
            side,
            price,
            base_quantity,
            quote_quantity_commission,
            timestamp,
        }
    }

    pub fn with_buy(
        price: Price,
        base_quantity: Quantity,
        quote_quantity_commission: Quantity,
    ) -> Self {
        Self::new(
            TradeSide::Buy,
            price,
            base_quantity,
            quote_quantity_commission,
            time::timestamp().as_millis(),
        )
    }

    pub fn with_sell(
        price: Price,
        base_quantity: Quantity,
        quote_quantity_commission: Quantity,
    ) -> Self {
        Self::new(
            TradeSide::Sell,
            price,
            base_quantity,
            quote_quantity_commission,
            time::timestamp().as_millis(),
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Deserialize)]
pub enum TradeSide {
    #[serde(rename = "BUY")]
    Buy,

    #[serde(rename = "SELL")]
    Sell,
}

pub trait Ploy {
    type Params;

    fn trap<FQ>(params: Self::Params) -> Vec<Position>;
}

impl Position {
    pub fn min_profit_trades<B, S>(&self, buy: &B, sell: &S) -> Option<Vec<Trade>>
    where
        B: Fn(&Price, &QuoteQuantity) -> Option<BaseQuantity>,
        S: Fn(&Price, &BaseQuantity) -> Option<QuoteQuantity>,
    {
        let mut result = Vec::with_capacity(3);
        let buying_price = self.max_buying_price();
        let selling_price = self.min_selling_price();

        if buying_price <= &Price::ZERO
            || selling_price <= &Price::ZERO
            || &self.base_quantity < &Price::ZERO
            || &self.quote_quantity < &Price::ZERO
        {
            return None;
        };

        let quote_quantity = if self.is_short() {
            self.quote_quantity
        } else {
            let exp_selling_quote_quantity = selling_price * self.base_quantity;
            let act_selling_quote_quantity = sell(selling_price, &self.base_quantity)?;

            let quote_quantity_commission = exp_selling_quote_quantity - act_selling_quote_quantity;

            result.push(Trade::with_sell(
                selling_price.clone(),
                self.base_quantity,
                quote_quantity_commission,
            ));

            act_selling_quote_quantity + self.quote_quantity
        };

        let base_quantity = {
            let exp_buying_base_quantity = quote_quantity / buying_price;
            let act_buying_base_quantity = buy(buying_price, &quote_quantity)?;

            let quote_quantity_commission =
                (exp_buying_base_quantity - act_buying_base_quantity) * buying_price;

            result.push(Trade::with_buy(
                buying_price.clone(),
                act_buying_base_quantity,
                quote_quantity_commission,
            ));

            act_buying_base_quantity
        };

        let _quote_quantity = {
            let exp_selling_quote_quantity = selling_price * base_quantity;
            let act_selling_quote_quantity = sell(selling_price, &base_quantity)?;

            let quote_quantity_commission = exp_selling_quote_quantity - act_selling_quote_quantity;

            result.push(Trade::with_sell(
                selling_price.clone(),
                base_quantity,
                quote_quantity_commission,
            ));

            act_selling_quote_quantity
        };

        return Some(result);
    }

    pub fn is_short(&self) -> bool {
        self.base_quantity.is_zero()
    }

    pub fn max_buying_price(&self) -> &Price {
        let mut max_buy_price = &Price::ZERO;
        for range in self.buying_prices.iter() {
            if range.max() > max_buy_price {
                max_buy_price = range.max();
            }
        }

        max_buy_price
    }

    pub fn min_selling_price(&self) -> &Price {
        let mut min_sell_price = &Price::MAX;
        for range in self.selling_prices.iter() {
            if range.min() < min_sell_price {
                min_sell_price = range.min();
            }
        }

        min_sell_price
    }
}

#[cfg(test)]
mod tests {
    use crate::services::binance::{
        math::Range,
        ploy::Trade,
        types::{BaseQuantity, Decimal, Price, QuoteQuantity},
    };

    use super::Position;

    fn buy(commission: Decimal) -> impl Fn(&Price, &QuoteQuantity) -> Option<BaseQuantity> {
        move |price: &Price, quantity: &QuoteQuantity| {
            if price > &Decimal::ZERO {
                return Some((quantity / price) * (Decimal::ONE - commission));
            }

            None
        }
    }

    fn sell(commission: Decimal) -> impl Fn(&Price, &BaseQuantity) -> Option<QuoteQuantity> {
        move |price: &Price, quantity: &BaseQuantity| {
            if price > &Decimal::ZERO {
                return Some((quantity * price) * (Decimal::ONE - commission));
            }

            None
        }
    }

    fn dec(value: &str) -> Decimal {
        use std::str::FromStr;
        Decimal::from_str(value).unwrap()
    }

    #[test]
    fn test_min_profit_trades_with_short() {
        let position = Position {
            buying_prices: vec![Range(dec("30"), dec("50"))],
            selling_prices: vec![Range(dec("200"), dec("250"))],
            base_quantity: dec("0.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .min_profit_trades(&buy(dec("0")), &sell(dec("0")))
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_buy(dec("50"), dec("0.4"), dec("0")),
                Trade::with_sell(dec("200"), dec("0.4"), dec("0"))
            ]
        );

        let trades = position
            .min_profit_trades(&buy(dec("0.001")), &sell(dec("0.001")))
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_buy(dec("50"), dec("0.3996"), dec("0.02")),
                Trade::with_sell(dec("200"), dec("0.3996"), dec("0.0799200"))
            ]
        );
    }

    #[test]
    fn test_min_profit_trades() {
        let position = Position {
            buying_prices: vec![Range(dec("30"), dec("80"))],
            selling_prices: vec![Range(dec("210"), dec("250"))],
            base_quantity: dec("5.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .min_profit_trades(&buy(dec("0")), &sell(dec("0")))
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_sell(dec("210"), dec("5"), dec("0")),
                Trade::with_buy(dec("80"), dec("13.375"), dec("0")),
                Trade::with_sell(dec("210"), dec("13.375"), dec("0"))
            ]
        );
    }

    #[test]
    fn test_min_profit_trades_with_mulit_prices() {
        let position = Position {
            buying_prices: vec![Range(dec("30"), dec("80")), Range(dec("90"), dec("100"))],
            selling_prices: vec![Range(dec("210"), dec("250")), Range(dec("205"), dec("200"))],
            base_quantity: dec("5.0"),
            quote_quantity: dec("20.0"),
        };

        let trades = position
            .min_profit_trades(&buy(dec("0")), &sell(dec("0")))
            .unwrap();
        assert_eq!(
            trades,
            vec![
                Trade::with_sell(dec("200"), dec("5"), dec("0")),
                Trade::with_buy(dec("100"), dec("10.2"), dec("0")),
                Trade::with_sell(dec("200"), dec("10.2"), dec("0"))
            ]
        );
    }

    impl PartialEq for Trade {
        fn eq(&self, other: &Self) -> bool {
            self.side == other.side
                && self.price == other.price
                && self.base_quantity == other.base_quantity
                && self.quote_quantity_commission == other.quote_quantity_commission
        }
    }
}
