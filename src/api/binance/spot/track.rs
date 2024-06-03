pub mod post {
    pub const PATH: &str = "/binance/spot/track";

    pub mod handler {
        use std::str::FromStr;

        use plot::trade::evaluate::Evaluater;
        use plot::trade::Executor;
        use plot::types::Decimal;

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use crate::services::binance::{client, BinanceSpotTest};

        use super::models::{Payload, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(mut p): Json<Payload>) -> ResponseResult<ResponseBody> {
            let client = client()?;
            let normal = {
                let mut info = client.exchange_info(&p.symbol).await?;
                match info.symbols.pop() {
                    Some(v) => v,
                    None => {
                        return Err(Response::bad_request(format!(
                            "symbol {} exchange info not found",
                            p.symbol
                        )))
                    }
                }
            };

            let commission = p.commission.unwrap_or(Decimal::from_str("0.001").unwrap());

            let agent = BinanceSpotTest::new(normal, commission);

            let mut trades = Vec::new();
            for price in p.prices.iter() {
                trades.extend(p.positions.trap(&agent, price).await.unwrap_or_default());
            }

            Ok(Response::ok(ResponseBody {
                evaluate: trades.evaluate().await,
                trades,
                positions: p.positions,
            }))
        }
    }

    pub mod models {
        use binance::types::Symbol;
        use plot::{
            trade::{evaluate::Evaluate, position::Position, Trade},
            types::{Decimal, Price},
        };
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub symbol: Symbol,
            pub commission: Option<Decimal>,
            pub positions: Vec<Position>,
            pub prices: Vec<Price>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub evaluate: Evaluate,
            pub trades: Vec<Trade>,
            pub positions: Vec<Position>,
        }
    }
}
