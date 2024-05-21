pub mod commission {
    pub mod post_commission {
        pub const PATH: &str = "/binance/spot/account/commission";

        use binance::types::{SpotCommission, Symbol};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Payload {
            api_key: String,
            secret_key: String,
            symbol: Symbol,
        }

        pub type Reply = SpotCommission;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
            let client = client_with_sign(p.api_key, p.secret_key)?;

            let result = client.spot_commission(&p.symbol).await?;

            Ok(Response::ok(result))
        }
    }
}

pub mod asset {
    pub mod post_asset {
        pub const PATH: &str = "/binance/spot/account/asset";

        use binance::types::{Asset, UserAsset};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Payload {
            api_key: String,
            secret_key: String,
            asset: Option<Asset>,
        }

        pub type Reply = Vec<UserAsset>;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
            let client = client_with_sign(p.api_key, p.secret_key)?;

            let asset = match &p.asset {
                Some(v) => Some(v),
                None => None,
            };

            let result = client.user_asset(asset, Some(false), None).await?;

            Ok(Response::ok(result))
        }
    }
}
