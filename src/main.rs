use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use harmony::api;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let address = env::var("ADDRESS").unwrap_or("127.0.0.1:3000".into());

    let cert_path = env::var("CERT_PATH");
    let key_path = env::var("KEY_PATH");

    let router = {
        let state = Arc::new(api::State::new().await);

        let router = Router::new();
        let router = router.merge(api::general::router(state.clone()));

        #[cfg(feature = "crypto")]
        let router = router.merge(api::crypto::router(state.clone()));

        #[cfg(feature = "binance")]
        let router = router.merge(api::binance::router(state.clone()));

        router
    };

    let addr: SocketAddr = address.parse().unwrap();

    if cert_path.is_ok() && key_path.is_ok() {
        let config = RustlsConfig::from_pem_file(cert_path.unwrap(), key_path.unwrap())
            .await
            .unwrap();

        axum_server::bind_rustls(addr, config)
            .serve(router.into_make_service())
            .await
            .unwrap();
    } else {
        axum_server::bind(addr)
            .serve(router.into_make_service())
            .await
            .unwrap();
    }
}
