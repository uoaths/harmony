pub mod post {
    pub const PATH: &str = "/binance/spot/order/info";

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
            let order = client.spot_order_info(&p.symbol, p.order_id, None).await?;
            let trades = client.spot_trade(&p.symbol, p.order_id, None).await?;

            Ok(Response::ok(ResponseBody { order, trades }))
        }
    }

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use binance::types::{OrderInfo, Symbol, Trade};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub api_key: String,
            pub secret_key: String,
            pub order_id: i64,
            pub symbol: Symbol,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub order: OrderInfo,
            pub trades: Vec<Trade>,
        }
    }
}
