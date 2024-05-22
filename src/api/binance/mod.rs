pub mod spot;

#[cfg(feature = "server")]
pub fn router(state: std::sync::Arc<crate::api::State>) -> axum::Router {
    use axum::routing::{get, post};
    use axum::Router;

    let router_price = Router::new().route(
        spot::price::get::PATH,
        get(spot::price::get::handler::handler),
    );

    let router_normal = Router::new().route(
        spot::normal::get::PATH,
        get(spot::normal::get::handler::handler),
    );

    let router_account = Router::new()
        .route(
            spot::account::asset::post::PATH,
            post(spot::account::asset::post::handler::handler),
        )
        .route(
            spot::account::commission::post::PATH,
            post(spot::account::commission::post::handler::handler),
        );

    let router_order = Router::new()
        .route(
            spot::order::post::PATH,
            post(spot::order::post::handler::handler),
        )
        .route(
            spot::order::buy::post::PATH,
            post(spot::order::buy::post::handler::handler),
        )
        .route(
            spot::order::sell::post::PATH,
            post(spot::order::sell::post::handler::handler),
        )
        .route(
            spot::order::info::post::PATH,
            post(spot::order::info::post::handler::handler),
        )
        .route(
            spot::order::trades::post::PATH,
            post(spot::order::trades::post::handler::handler),
        );

    Router::new()
        .merge(router_account)
        .merge(router_normal)
        .merge(router_order)
        .merge(router_price)
        .with_state(state)
}
