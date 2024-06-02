pub mod buy;
pub mod info;
pub mod sell;
pub mod trades;

pub mod post {
    pub const PATH: &str = "/binance/spot/order";

    pub mod handler {
        use std::str::FromStr;

        use plot::trade::Executor;
        use plot::types::Decimal;

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::{client_with_sign, BinanceSpot};

        use super::models::{Order, Payload, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(mut p): Json<Payload>) -> ResponseResult<ResponseBody> {
            let client = client_with_sign(p.api_key, p.secret_key)?;

            let price = {
                let price = client.price(&p.symbol).await?.price;
                Decimal::from_str(&price).unwrap()
            };

            let normal = {
                let mut info = client.exchange_info(&p.symbol).await?;
                match info.symbols.pop() {
                    Some(v) => v,
                    None => return Err(Response::bad_request("exchange info not found".into())),
                }
            };

            let mut order = Vec::new();
            let spot_agent = BinanceSpot::new(normal, client);
            for position in p.positions.iter_mut() {
                let trades = position.trap(&spot_agent, &price).await.unwrap_or_default();
                order.push(Order {
                    order_id: 1,
                    symbol: p.symbol.clone(),
                    trades,
                });
            }

            Ok(Response::ok(ResponseBody {
                positions: p.positions,
                order,
                price,
                symbol: p.symbol,
            }))
        }
    }

    pub mod models {
        use binance::types::Symbol;
        use plot::{trade::position::Position, trade::Trade, types::Price};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub api_key: String,
            pub secret_key: String,
            pub symbol: Symbol,
            pub positions: Vec<Position>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub symbol: Symbol,
            pub price: Price,
            pub positions: Vec<Position>,
            pub order: Vec<Order>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Order {
            pub order_id: i64,
            pub symbol: Symbol,
            pub trades: Vec<Trade>,
        }
    }
}
