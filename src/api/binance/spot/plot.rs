pub mod post {
    pub const PATH: &str = "/binance/spot/plot";

    pub mod handler {
        use std::str::FromStr;

        use ploy::types::Decimal;

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use crate::services::binance::{client, plot};

        use super::models::{Payload, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<ResponseBody> {
            use ploy::Ploy;

            let client = client()?;
            let exchange = client.exchange_info(&p.symbol).await?;
            let normal = match exchange.symbols.first() {
                Some(v) => v,
                None => return Err(Response::bad_request("exchange info not found".into())),
            };

            let commission = p.commission.unwrap_or(Decimal::from_str("0.001").unwrap());
            let mut positions = Vec::new();

            if let Some(grid) = p.grid {
                positions = grid.trap()
            }

            Ok(Response::ok(ResponseBody {
                profit: plot::profit(&positions, normal, &commission),
                positions: positions,
            }))
        }
    }

    pub mod models {
        use binance::types::Symbol;
        use ploy::{
            plot::grid::Grid,
            position::Position,
            types::{Decimal, QuoteQuantity},
        };
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub symbol: Symbol,
            pub commission: Option<Decimal>,
            pub grid: Option<Grid>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub profit: Option<QuoteQuantity>,
            pub positions: Vec<Position>,
        }
    }
}
