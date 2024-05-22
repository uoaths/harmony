pub mod post {
    pub const PATH: &str = "/binance/spot/order/trades";

    #[cfg(feature = "server-api-handler")]
    pub mod handler {
        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        use super::models::{Payload, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<ResponseBody> {
            let client = client_with_sign(p.api_key, p.secret_key)?;
            let result = client
                .spot_trades(
                    &p.symbol,
                    p.order_id,
                    p.start_time,
                    p.end_time,
                    p.from_id,
                    p.limit,
                    None,
                )
                .await?;

            Ok(Response::ok(result))
        }
    }

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use binance::types::{Symbol, Trade};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub api_key: String,
            pub secret_key: String,
            pub symbol: Symbol,
            pub order_id: Option<i64>,
            pub start_time: Option<u128>,
            pub end_time: Option<u128>,
            pub from_id: Option<i64>,
            pub limit: Option<u16>,
        }

        pub type ResponseBody = Vec<Trade>;
    }
}
