mod health;

mod get {
    pub const PATH: &str = "/";

    #[cfg(feature = "server-api-handler")]
    pub mod handler {
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use super::models::ResponseBody;

        #[tracing::instrument(skip(c))]
        pub async fn handler(c: Trip) -> ResponseResult<ResponseBody> {
            let response = Response::ok(ResponseBody {
                timestamp: c.timestamp_millis(),
            });

            Ok(response)
        }
    }

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub timestamp: u128,
        }
    }
}

#[cfg(feature = "server")]
pub fn router(state: std::sync::Arc<crate::api::State>) -> axum::Router {
    use axum::{routing::get, Router};

    Router::new()
        .route(get::PATH, get(get::handler::handler))
        .route(health::get::PATH, get(health::get::handler::handler))
        .with_state(state)
}
