mod token;
mod uniswap;

pub fn router(state: std::sync::Arc<crate::api::State>) -> axum::Router {
    use axum::{routing::get, Router};

    Router::new()
        .route(
            uniswap::get_uniswap_v3_pool::PATH,
            get(uniswap::get_uniswap_v3_pool::handler),
        )
        .route(
            token::get_token_erc20::PATH,
            get(token::get_token_erc20::handler),
        )
        .route(
            token::get_token_price::PATH,
            get(token::get_token_price::handler),
        )
        .with_state(state)
}
