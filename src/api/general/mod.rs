pub fn router(state: std::sync::Arc<crate::api::State>) -> axum::Router {
    use axum::{routing::get, Router};

    Router::new()
        .route(get_root::PATH, get(get_root::handler))
        .route(get_health::PATH, get(get_health::handler))
        .with_state(state)
}

mod get_health {
    pub const PATH: &str = "/health";

    use serde::Serialize;

    use crate::api::http::response::Response;
    use crate::api::http::trip::Trip;

    #[derive(Serialize)]
    pub struct Reply {
        timestamp: u128,
    }

    #[tracing::instrument(skip(c))]
    pub async fn handler(c: Trip) -> Response<Reply> {
        Response::ok(Reply {
            timestamp: c.timestamp_millis(),
        })
    }
}

mod get_root {
    pub const PATH: &str = "/";

    use serde::Serialize;

    use crate::api::http::response::{Response, ResponseResult};
    use crate::api::http::trip::Trip;

    #[derive(Serialize)]
    pub struct Reply {
        timestamp: u128,
    }

    #[tracing::instrument(skip(c))]
    pub async fn handler(c: Trip) -> ResponseResult<Reply> {
        let response = Response::ok(Reply {
            timestamp: c.timestamp_millis(),
        });

        Ok(response)
    }
}
