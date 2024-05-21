pub mod account;
pub mod order;

pub mod price {
    pub mod get_price {
        pub const PATH: &str = "/binance/spot/price";

        use binance::types::{Symbol, SymbolPrice};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Query;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Params {
            symbol: Option<Symbol>,
        }

        type Reply = Vec<SymbolPrice>;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Query(q): Query<Params>) -> ResponseResult<Reply> {
            let client = client()?;

            let result = match q.symbol {
                Some(v) => {
                    vec![client.price(&v).await?]
                }
                None => client.prices(None).await?,
            };

            Ok(Response::ok(result))
        }
    }
}

pub mod normal {
    pub mod get_normal {
        pub const PATH: &str = "/binance/spot/normal";

        use binance::types::{ExchangeInfo, Symbol};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Query;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use crate::services::binance::client;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Params {
            symbol: Symbol,
        }

        type Reply = ExchangeInfo;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Query(p): Query<Params>) -> ResponseResult<Reply> {
            let client = client()?;
            let result = client.exchange_info(&p.symbol).await?;

            Ok(Response::ok(result))
        }
    }
}
