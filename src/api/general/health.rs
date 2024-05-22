pub mod get {
    pub const PATH: &str = "/health";

    #[cfg(feature = "server-api-handler")]
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

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub timestamp: u128,
        }
    }
}
