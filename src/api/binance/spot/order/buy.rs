pub mod post {
    pub const PATH: &str = "/binance/spot/order/buy";

    pub mod handler {
        use binance::types::OrderSide;

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        use super::models::{Payload, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<ResponseBody> {
            let client = client_with_sign(p.api_key, p.secret_key)?;

            let result = client
                .spot_market_order_with_quote(
                    &p.symbol,
                    OrderSide::Buy,
                    &p.quote_quantity.to_string(),
                    None,
                )
                .await?;

            Ok(Response::ok(result))
        }
    }

    pub mod models {
        use binance::types::{OrderResponseFull, Symbol};
        use plot::types::Quantity;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub api_key: String,
            pub secret_key: String,
            pub symbol: Symbol,
            pub quote_quantity: Quantity,
        }

        pub type ResponseBody = OrderResponseFull;
    }
}
