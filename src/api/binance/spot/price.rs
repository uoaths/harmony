pub mod get {
    pub const PATH: &str = "/binance/spot/price";

    #[cfg(feature = "server-api-handler")]
    pub mod handler {
        use crate::api::http::request::Query;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client;

        use super::models::{Params, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Query(q): Query<Params>) -> ResponseResult<ResponseBody> {
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

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use binance::types::{Symbol, SymbolPrice};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Params {
            pub symbol: Option<Symbol>,
        }

        pub type ResponseBody = Vec<SymbolPrice>;
    }
}
