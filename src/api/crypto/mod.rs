pub mod token;
pub mod uniswap;

#[cfg(feature = "server")]
pub fn router(state: std::sync::Arc<crate::api::State>) -> axum::Router {
    use axum::routing::get;

    axum::Router::new()
        .route(uniswap::get::PATH, get(uniswap::get::handler::handler))
        .route(
            token::erc20::get::PATH,
            get(token::erc20::get::handler::handler),
        )
        .route(
            token::price::get::PATH,
            get(token::price::get::handler::handler),
        )
        .with_state(state)
}
