pub mod post {
    pub const PATH: &str = "/binance/spot/account/commission";

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

            let result = client.spot_commission(&p.symbol).await?;

            Ok(Response::ok(result))
        }
    }

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use binance::types::{SpotCommission, Symbol};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub api_key: String,
            pub secret_key: String,
            pub symbol: Symbol,
        }

        pub type ResponseBody = SpotCommission;
    }
}
