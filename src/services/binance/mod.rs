pub mod filter;

use binance::prelude::{Client, ClientBuilder};
use std::{error::Error, time::Duration};

pub fn client() -> Result<Client, Box<dyn Error>> {
    let result = ClientBuilder::new().build()?;

    Ok(result)
}

pub fn client_with_sign(api_key: String, secret_key: String) -> Result<Client, Box<dyn Error>> {
    let result = ClientBuilder::new()
        .set_api_key(api_key)
        .set_secret_key(secret_key)
        .set_timeout(Duration::from_secs(5))
        .build()?;

    Ok(result)
}

pub mod order {
    use binance::{
        error::ClientError,
        prelude::Client,
        types::{OrderResponseFull, OrderSide, Symbol},
    };
    use ploy::types::Quantity;

    pub async fn place_buying_market_order_with_quote(
        client: &Client,
        symbol: &Symbol,
        quote_quantity: &Quantity,
    ) -> Result<OrderResponseFull, ClientError> {
        client
            .spot_market_order_with_quote(
                &symbol,
                OrderSide::Buy,
                &quote_quantity.to_string(),
                None,
            )
            .await
    }

    pub async fn place_selling_market_order_with_base(
        client: &Client,
        symbol: &Symbol,
        base_quantity: &Quantity,
    ) -> Result<OrderResponseFull, ClientError> {
        client
            .spot_market_order_with_base(&symbol, OrderSide::Sell, &base_quantity.to_string(), None)
            .await
    }
}

pub mod plot {
    use binance::types::SymbolInfo;
    use ploy::position::{Position, Trade};
    use ploy::types::{BaseQuantity, Decimal, Price, QuoteQuantity};

    use super::filter;

    fn buy(
        commission: Decimal,
        norms: SymbolInfo,
    ) -> impl Fn(&Price, &QuoteQuantity) -> Option<BaseQuantity> {
        move |price: &Price, quantity: &QuoteQuantity| {
            if price > &Decimal::ZERO {
                let quantity =
                    &filter::spot::quote_quantity::correct(&norms, price, quantity).ok()?;
                filter::spot::quote_quantity::filter(&norms, price, quantity).ok()?;
                return Some((quantity / price) * (Decimal::ONE - commission));
            }

            None
        }
    }

    fn sell(
        commission: Decimal,
        norms: SymbolInfo,
    ) -> impl Fn(&Price, &BaseQuantity) -> Option<QuoteQuantity> {
        move |price: &Price, quantity: &BaseQuantity| {
            if price > &Decimal::ZERO {
                let quantity =
                    &filter::spot::base_quantity::correct(&norms, price, quantity).ok()?;
                filter::spot::base_quantity::filter(&norms, price, quantity).ok()?;
                return Some((quantity * price) * (Decimal::ONE - commission));
            }

            None
        }
    }

    pub fn profit(
        positions: &Vec<Position>,
        norms: &SymbolInfo,
        commission: &Decimal,
    ) -> Option<QuoteQuantity> {
        let mut trades = Vec::new();
        for position in positions {
            let t = position.min_profit_trades(
                &buy(*commission, norms.clone()),
                &sell(*commission, norms.clone()),
            )?;
            trades.extend(t);
        }

        Some(Trade::profit(&trades).1)
    }
}
