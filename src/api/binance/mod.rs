mod spot;

#[rustfmt::skip]
pub fn router(state: std::sync::Arc<crate::api::State>) -> axum::Router {
    use axum::routing::{get, post};
    use axum::Router;

    let router_price = Router::new()
    .route(spot::price::get_price::PATH, get(spot::price::get_price::handler));

    let router_normal = Router::new()
    .route(spot::normal::get_normal::PATH, get(spot::normal::get_normal::handler));

    let router_account = Router::new()
    .route(spot::account::asset::post_asset::PATH,           post(spot::account::asset::post_asset::handler))
    .route(spot::account::commission::post_commission::PATH, post(spot::account::commission::post_commission::handler));

    let router_order = Router::new()
    .route(spot::order::post_order::PATH,          post(spot::order::post_order::handler))
    .route(spot::order::buy::post_buy::PATH,       post(spot::order::buy::post_buy::handler))
    .route(spot::order::sell::post_sell::PATH,     post(spot::order::sell::post_sell::handler))
    .route(spot::order::info::post_info::PATH,     post(spot::order::info::post_info::handler))
    .route(spot::order::trades::post_trades::PATH, post(spot::order::trades::post_trades::handler));

    Router::new()
    .merge(router_account)
    .merge(router_normal)
    .merge(router_order)
    .merge(router_price)
    .with_state(state)
}
