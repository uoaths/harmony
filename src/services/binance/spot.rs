use binance::{
    error::ClientError,
    prelude::Client,
    types::{OrderResponseFull, OrderSide, Symbol},
};

use super::types::Quantity;

pub async fn place_buying_market_order_with_quote(
    client: &Client,
    symbol: &Symbol,
    quote_quantity: &Quantity,
) -> Result<OrderResponseFull, ClientError> {
    client
        .spot_market_order_with_quote(&symbol, OrderSide::Buy, &quote_quantity.to_string(), None)
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
