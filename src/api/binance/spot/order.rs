pub mod post_order {
    pub const PATH: &str = "/binance/spot/order";

    use binance::types::{OrderSide, Quantity, Symbol};
    use serde::{Deserialize, Serialize};

    use crate::api::http::request::Json;
    use crate::api::http::response::ResponseResult;
    use crate::api::http::trip::Trip;
    use crate::services::binance::client_with_sign;
    use crate::services::binance::math::Range;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Payload {
        api_key: String,
        secret_key: String,
        symbol: Symbol,
        positions: Vec<Position>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Reply {
        positions: Vec<Position>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Position {
        selling_price: Range,
        buying_price: Range,
        base_quantity: Quantity,
        quote_quantity: Quantity,
    }

    #[tracing::instrument(skip(c))]
    pub async fn handler(c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
        let client = client_with_sign(p.api_key, p.secret_key)?;

        let symbol = p.symbol;
        let price = client.price(&symbol).await.unwrap().price;

        let price = match price.parse() {
            Ok(num) => num,
            Err(e) => {
                todo!()
            }
        };

        for position in p.positions.iter() {
            if position.buying_price.is_within(price) {
                let buying_order = client
                    .spot_market_order_with_quote(
                        &symbol,
                        OrderSide::Buy,
                        &position.quote_quantity,
                        None,
                    )
                    .await
                    .unwrap();

                break;
            }

            if position.selling_price.is_within(price) {
                let selling_order = client
                    .spot_market_order_with_base(
                        &symbol,
                        OrderSide::Sell,
                        &position.base_quantity,
                        None,
                    )
                    .await
                    .unwrap();

                break;
            }
        }

        todo!();
    }
}
