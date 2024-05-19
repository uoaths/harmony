pub mod norm {
    pub mod get_norm {
        pub const PATH: &str = "/binance/market/norm";

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
