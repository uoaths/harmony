pub mod get {
    pub const PATH: &str = "/health";

    pub mod handler {
        use crate::api::http::response::Response;
        use crate::api::http::trip::Trip;

        use super::models::ResponseBody;

        #[tracing::instrument(skip(c))]
        pub async fn handler(c: Trip) -> Response<ResponseBody> {
            Response::ok(ResponseBody {
                timestamp: c.timestamp_millis(),
            })
        }
    }

    pub mod models {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub timestamp: u128,
        }
    }
}
