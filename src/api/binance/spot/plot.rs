pub mod post {
    pub const PATH: &str = "/binance/spot/plot";

    pub mod handler {
        use std::str::FromStr;

        use plot::strategy::Strategy;
        use plot::trade::evaluate::Evaluater;
        use plot::types::Decimal;

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use crate::services::binance::{client, BinanceSpotTest};

        use super::models::{Analyzer, Payload, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<ResponseBody> {
            let client = client()?;
            let normal = {
                let mut info = client.exchange_info(&p.symbol).await?;
                match info.symbols.pop() {
                    Some(v) => v,
                    None => return Err(Response::bad_request("exchange info not found".into())),
                }
            };

            let commission = p.commission.unwrap_or(Decimal::from_str("0.001").unwrap());

            let positions = {
                let mut positions = Vec::new();
                if let Some(grid) = p.grid {
                    positions = grid.assign_position()
                }

                if let Some(grid) = p.grid_percent {
                    positions = grid.assign_position()
                }

                positions
            };

            let analyzer = {
                let mut analyzer = Vec::new();
                let positions = positions.clone();
                let spot_agent = BinanceSpotTest::new(normal, commission);

                for mut position in positions.into_iter() {
                    let trades = position.min_profit_trades(&spot_agent).await?;
                    analyzer.push(Analyzer {
                        evaluate: trades.evaluate().await,
                        trades,
                        position,
                    });
                }

                analyzer
            };

            Ok(Response::ok(ResponseBody {
                positions,
                analyzer,
            }))
        }
    }

    pub mod models {
        use binance::types::Symbol;
        use plot::{
            strategy::{grid::Grid, grid_percent::GridPercent},
            trade::{evaluate::Evaluate, position::Position, Trade},
            types::Decimal,
        };
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub symbol: Symbol,
            pub commission: Option<Decimal>,
            pub grid: Option<Grid>,
            pub grid_percent: Option<GridPercent>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub analyzer: Vec<Analyzer>,
            pub positions: Vec<Position>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Analyzer {
            pub evaluate: Evaluate,
            pub trades: Vec<Trade>,
            pub position: Position,
        }
    }
}
