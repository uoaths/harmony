pub mod filter;

use std::{error::Error, time::Duration};
use binance::prelude::{Client, ClientBuilder};

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
