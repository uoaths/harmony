mod spot;

pub fn router(state: std::sync::Arc<crate::api::State>) -> axum::Router {
    use axum::{
        routing::{get, post},
        Router,
    };

    Router::new()
        .route(
            spot::order::post_order::PATH,
            post(spot::order::post_order::handler),
        )
        .route(
            spot::price::get_price::PATH,
            get(spot::price::get_price::handler),
        )
        .route(
            spot::asset::post_asset::PATH,
            post(spot::asset::post_asset::handler),
        )
        .route(
            spot::commission::post_commission::PATH,
            post(spot::commission::post_commission::handler),
        )
        .route(
            spot::order::buy::post_buy::PATH,
            post(spot::order::buy::post_buy::handler),
        )
        .route(
            spot::order::sell::post_sell::PATH,
            post(spot::order::sell::post_sell::handler),
        )
        .with_state(state)
}
