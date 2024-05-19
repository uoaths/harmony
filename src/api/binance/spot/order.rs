pub mod post_order {
    pub const PATH: &str = "/binance/spot/order";

    use binance::types::{OrderSide, Price, Quantity, Symbol};
    use serde::{Deserialize, Serialize};

    use crate::api::http::request::Json;
    use crate::api::http::response::ResponseResult;
    use crate::api::http::trip::Trip;
    use crate::services::binance::client_with_sign;
    use crate::services::binance::math::Range;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Payload {
        api_key: String,
        secret_key: String,
        symbol: Symbol,
        positions: Vec<Position>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Reply {
        order: Vec<Order>,
        positions: Vec<Position>,
    }

    // Buy  Side: Converts quote currency to base currency.
    // Sell Side: Converts base currency to quote currency.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Order {
        unique: String,
        symbol: Symbol,

        /// The price of the base currency expressed in terms of the quote currency.
        price: Price,

        /// The quantity of the quote currency involved in the trade.
        quantity: Quantity,

        /// The side of the order: BUY or SELL.
        side: OrderSide,

        /// A timestamp marking when the order was placed, milliseconds.
        timestamp: u128,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Position {
        selling_price: Range,
        buying_price: Range,
        base_quantity: Quantity,
        quote_quantity: Quantity,
    }

    #[tracing::instrument(skip(c))]
    pub async fn handler(c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
        let client = client_with_sign(p.api_key, p.secret_key)?;

        let symbol = p.symbol;
        let price = client.price(&symbol).await?.price;

        let price = match price.parse() {
            Ok(num) => num,
            Err(e) => {
                todo!()
            }
        };

        for position in p.positions.iter() {
            if position.buying_price.is_within(price) {
                let buying_order = client
                    .spot_market_order_with_quote(
                        &symbol,
                        OrderSide::Buy,
                        &position.quote_quantity,
                        None,
                    )
                    .await
                    .unwrap();

                break;
            }

            if position.selling_price.is_within(price) {
                // filter: LOT_SIZE
                // maxQty >= base_quantity >= minQty;
                // base_quantity % stepSize == 0;
                //
                // leave_quantity = base_quantity % stepSize;
                // sell_base_quantity = base_quantity - leave_quantity;

                // filter: NOTIONAL
                // maxNotional >= base_quantity * price >= minNotional
                let selling_order = client
                    .spot_market_order_with_base(
                        &symbol,
                        OrderSide::Sell,
                        &position.base_quantity,
                        None,
                    )
                    .await
                    .unwrap();

                break;
            }
        }

        todo!();
    }
}

pub mod buy {
    pub mod post_buy {
        pub const PATH: &str = "/binance/spot/order/buy";

        use binance::types::{OrderResponseFull, OrderSide, Quantity, Symbol};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use crate::services::binance::client_with_sign;
        use crate::services::binance::filter::spot_market::quote_quantity;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Payload {
            api_key: String,
            secret_key: String,
            symbol: Symbol,
            quote_quantity: Quantity,
        }

        type Reply = OrderResponseFull;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
            let client = client_with_sign(p.api_key, p.secret_key)?;

            // Filter quote quantity
            let quote_quantity = {
                let price = client.price(&p.symbol).await?.price;
                let norm = client.exchange_info(&p.symbol).await?;
                let symbol_norm = match norm.symbols.first() {
                    Some(v) => v,
                    None => return Err(Response::bad_request("exchange info not found".into())),
                };

                quote_quantity::filter(&price, &p.quote_quantity, &symbol_norm.filters)?
            };

            let result = client
                .spot_market_order_with_quote(&p.symbol, OrderSide::Buy, &quote_quantity, None)
                .await?;

            Ok(Response::ok(result))
        }
    }
}

pub mod sell {
    pub mod post_sell {
        pub const PATH: &str = "/binance/spot/order/sell";

        use binance::types::{OrderResponseFull, OrderSide, Quantity, Symbol};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use crate::services::binance::client_with_sign;
        use crate::services::binance::filter::spot_market::base_quantity;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Payload {
            api_key: String,
            secret_key: String,
            symbol: Symbol,
            base_quantity: Quantity,
        }

        type Reply = OrderResponseFull;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
            let client = client_with_sign(p.api_key, p.secret_key)?;

            // Filter base quantity
            let base_quantity = {
                let price = client.price(&p.symbol).await?.price;
                let norm = client.exchange_info(&p.symbol).await?;
                let symbol_norm = match norm.symbols.first() {
                    Some(v) => v,
                    None => return Err(Response::bad_request("exchange info not found".into())),
                };

                base_quantity::filter(&price, &p.base_quantity, &symbol_norm.filters)?
            };

            let result = client
                .spot_market_order_with_base(&p.symbol, OrderSide::Sell, &base_quantity, None)
                .await?;

            Ok(Response::ok(result))
        }
    }
}

pub mod check {
    pub mod post_check {
        pub const PATH: &str = "/binance/spot/order/check";

        use binance::types::{OrderInfo, Symbol};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Payload {
            api_key: String,
            secret_key: String,
            id: i64,
            symbol: Symbol,
        }

        type Reply = OrderInfo;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
            let client = client_with_sign(p.api_key, p.secret_key)?;
            let result = client.spot_order_info(&p.symbol, p.id, None).await?;

            Ok(Response::ok(result))
        }
    }
}

pub mod trades {
    pub mod post_trades {
        pub const PATH: &str = "/binance/spot/trades";

        use binance::types::{Symbol, Trade};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Payload {
            api_key: String,
            secret_key: String,
            symbol: Symbol,
            order_id: Option<i64>,
            start_time: Option<u128>,
            end_time: Option<u128>,
            from_id: Option<i64>,
            limit: Option<u16>,
        }

        type Reply = Vec<Trade>;

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
            let client = client_with_sign(p.api_key, p.secret_key)?;
            let result = client
                .spot_trades(
                    &p.symbol,
                    p.order_id,
                    p.start_time,
                    p.end_time,
                    p.from_id,
                    p.limit,
                    None,
                )
                .await?;

            Ok(Response::ok(result))
        }
    }
}
