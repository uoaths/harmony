pub mod trip {
    use std::sync::Arc;

    use crate::services::time::timestamp;

    pub(crate) type Trip = axum::extract::State<Arc<State>>;

    pub struct State {}

    impl State {
        pub async fn new() -> Self {
            Self {}
        }

        pub fn timestamp_millis(&self) -> u128 {
            timestamp().as_millis()
        }
    }
}

pub mod response {
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use serde::Serialize;
    use serde_json::json;

    pub type ResponseResult<T> = Result<Response<T>, Response<()>>;

    #[derive(Serialize)]
    pub struct Response<T>
    where
        T: Serialize,
    {
        pub(crate) ok: bool,
        pub(crate) code: u16,
        pub(crate) data: Option<T>,
        pub(crate) message: Option<String>,
    }

    impl<T> Response<T>
    where
        T: Serialize,
    {
        pub fn new() -> Self {
            Self {
                ok: true,
                code: 200,
                data: None,
                message: None,
            }
        }

        pub fn ok(data: T) -> Self {
            let mut response = Self::new();
            response.data = Some(data);

            response
        }

        // pub fn forbidden(message: String) -> Self {
        //     let mut response = Self::new();
        //     response.code = 403;
        //     response.message = Some(message);

        //     response
        // }

        pub fn bad_request(message: String) -> Self {
            let mut response = Self::new();
            response.ok = false;
            response.code = 400;
            response.message = Some(message);

            response
        }

        // pub fn internal_error(message: String) -> Self {
        //     let mut response = Self::new();
        //     response.ok = false;
        //     response.code = 500;
        //     response.message = Some(message);

        //     response
        // }
    }

    impl<T> IntoResponse for Response<T>
    where
        T: Serialize,
    {
        fn into_response(self) -> axum::response::Response {
            let code = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let body = Json(json!(self));

            (code, body).into_response()
        }
    }

    mod from_general_error {
        use std::error::Error;

        use super::{Response, Serialize};

        impl<T> From<Box<dyn Error>> for Response<T>
        where
            T: Serialize,
        {
            fn from(value: Box<dyn Error>) -> Self {
                Self::bad_request(value.to_string())
            }
        }
    }

    #[cfg(feature = "crypto")]
    mod from_contract_error {
        use crate::services::crypto::contract::ContractError;

        use super::{Response, Serialize};

        impl<T> From<ContractError> for Response<T>
        where
            T: Serialize,
        {
            fn from(value: ContractError) -> Self {
                Self::bad_request(value.to_string())
            }
        }
    }

    #[cfg(feature = "binance")]
    mod from_binance_client_error {
        use binance::error::ClientError;

        use super::{Response, Serialize};

        impl<T> From<ClientError> for Response<T>
        where
            T: Serialize,
        {
            fn from(value: ClientError) -> Self {
                Self::bad_request(value.to_string())
            }
        }
    }

    #[cfg(feature = "binance")]
    mod from_binance_filter_error {
        use crate::services::binance::filter::error::SymbolFilterError;

        use super::{Response, Serialize};

        impl<T> From<SymbolFilterError> for Response<T>
        where
            T: Serialize,
        {
            fn from(value: SymbolFilterError) -> Self {
                Self::bad_request(value.to_string())
            }
        }
    }
}

pub mod request {
    use axum::async_trait;
    use axum::extract::Query as AxumQuery;
    use axum::extract::{rejection::JsonRejection, FromRequest, FromRequestParts, Request};
    use axum::http::request::Parts;
    use axum::Json as AxumJson;
    use serde::de::DeserializeOwned;

    // ===== Query =====
    #[derive(Debug, Clone)]
    pub struct Query<T>(pub T);

    #[async_trait]
    impl<T, S> FromRequestParts<S> for Query<T>
    where
        T: DeserializeOwned,
        S: Send + Sync,
    {
        type Rejection = super::response::Response<()>;

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            match AxumQuery::try_from_uri(&parts.uri) {
                Ok(value) => Ok(Self(value.0)),
                Err(rejection) => {
                    let response = super::response::Response::bad_request(rejection.body_text());

                    Err(response)
                }
            }
        }
    }

    // ===== JSON =====
    #[derive(Debug, Clone)]
    pub struct Json<T>(pub T);

    #[async_trait]
    impl<S, T> FromRequest<S> for Json<T>
    where
        AxumJson<T>: FromRequest<S, Rejection = JsonRejection>,
        S: Send + Sync,
    {
        type Rejection = super::response::Response<()>;

        async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
            match AxumJson::<T>::from_request(req, state).await {
                Ok(value) => Ok(Self(value.0)),
                Err(rejection) => {
                    let response = super::response::Response::bad_request(rejection.body_text());

                    Err(response)
                }
            }
        }
    }
}
