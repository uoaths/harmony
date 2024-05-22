pub mod post {
    pub const PATH: &str = "/binance/spot/account/asset";

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

            let asset = match &p.asset {
                Some(v) => Some(v),
                None => None,
            };

            let result = client.user_asset(asset, Some(false), None).await?;

            Ok(Response::ok(result))
        }
    }

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use binance::types::{Asset, UserAsset};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub api_key: String,
            pub secret_key: String,
            pub asset: Option<Asset>,
        }

        pub type ResponseBody = Vec<UserAsset>;
    }
}
