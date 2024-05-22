pub mod get {
    pub const PATH: &str = "/binance/spot/normal";

    pub mod handler {
        use crate::api::http::request::Query;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client;

        use super::models::{Params, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Query(p): Query<Params>) -> ResponseResult<ResponseBody> {
            let client = client()?;
            let result = client.exchange_info(&p.symbol).await?;

            Ok(Response::ok(result))
        }
    }

    pub mod models {
        use binance::types::{ExchangeInfo, Symbol};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Params {
            pub symbol: Symbol,
        }

        pub type ResponseBody = ExchangeInfo;
    }
}
