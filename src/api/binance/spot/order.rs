pub mod post_order {
    pub const PATH: &str = "/binance/spot/order";

    use binance::prelude::Client;
    use binance::types::{OrderSide, Symbol, SymbolInfo};
    use serde::{Deserialize, Serialize};

    use crate::api::http::request::Json;
    use crate::api::http::response::{Response, ResponseResult};
    use crate::api::http::trip::Trip;

    use crate::services::binance::client_with_sign;
    use crate::services::binance::math::Range;
    use crate::services::binance::types::{Decimal, Price, Quantity};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Payload {
        api_key: String,
        secret_key: String,
        symbol: Symbol,
        positions: Vec<Position>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Reply {
        positions: Vec<Position>,
        order: Vec<Order>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Order {
        order_id: i64,
        symbol: Symbol,
        side: OrderSide,
        timestamp: u128,
        trades: Vec<Trade>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Trade {
        price: Price,
        base_quantity: Quantity,
        quote_quantity_commission: Quantity,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Position {
        buying_price: Vec<Range>,
        selling_price: Vec<Range>,
        base_quantity: Quantity,
        quote_quantity: Quantity,
    }

    #[tracing::instrument(skip(_c))]
    pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
        let client = client_with_sign(p.api_key, p.secret_key)?;
        let norm = client.exchange_info(&p.symbol).await?;
        let price = client.price(&p.symbol).await?.price;
        let price = to_decimal(&price).unwrap();

        let norms = match norm.symbols.first() {
            Some(v) => v,
            None => return Err(Response::bad_request("exchange info not found".into())),
        };

        let mut order = Vec::with_capacity(p.positions.len());
        let mut positions = Vec::with_capacity(p.positions.len());

        for mut position in p.positions.into_iter() {
            if let Some(v) = sell(&client, &p.symbol, &price, &norms, &mut position).await {
                order.push(v);
            }

            if let Some(v) = buy(&client, &p.symbol, &price, &norms, &mut position).await {
                order.push(v)
            }

            positions.push(position)
        }

        Ok(Response::ok(Reply { positions, order }))
    }

    async fn buy(
        client: &Client,
        symbol: &Symbol,
        price: &Price,
        norms: &SymbolInfo,
        position: &mut Position,
    ) -> Option<Order> {
        use crate::services::binance::filter::spot_market::quote_quantity::{correct, filter};
        use crate::services::binance::math::is_within_price_ranges;
        use crate::services::binance::order::place_buying_market_order_with_quote as place;

        if !is_within_price_ranges(price, &position.buying_price) {
            return None;
        }

        // Filter the number of quotes to be bought.
        // If the filter is not successfully passed, None will be returned.
        let quote_quantity = correct(norms, price, &position.quote_quantity).ok()?;
        filter(norms, price, &quote_quantity).ok()?;

        // Buy the base quantity by the quoted quantity
        let order = place(client, symbol, &quote_quantity).await.ok()?;

        // Calculate the commission fee for the buy order and add it to the trade list
        let (trades, income_base_quantity) = {
            let mut trades = Vec::with_capacity(3);
            let mut trades_all_base_quantity = Decimal::ZERO;
            let mut trades_all_commission_base_quantity = Decimal::ZERO;

            for fill in order.fills.iter() {
                // fills fixedly returns the base quantity in the qty field
                let price = to_decimal(&fill.price).unwrap_or_default();
                let base_quantity = to_decimal(&fill.qty).unwrap_or_default();

                // In the buying direction, the quote commission of price * quantity needs to be calculated
                let commission = to_decimal(&fill.commission).unwrap_or_default();
                let quote_quantity_commission = price * commission;

                trades_all_base_quantity += base_quantity;
                trades_all_commission_base_quantity += commission;

                let trade = Trade {
                    price,
                    base_quantity,
                    quote_quantity_commission,
                };

                trades.push(trade)
            }

            let income_base_quantity =
                trades_all_base_quantity - trades_all_commission_base_quantity;

            (trades, income_base_quantity)
        };

        position.quote_quantity = position.quote_quantity - quote_quantity;
        position.base_quantity = position.base_quantity + income_base_quantity;

        return Some(Order {
            order_id: order.order_id,
            symbol: order.symbol,
            trades,
            side: order.side,
            timestamp: order.transact_time,
        });
    }

    async fn sell(
        client: &Client,
        symbol: &Symbol,
        price: &Price,
        norms: &SymbolInfo,
        position: &mut Position,
    ) -> Option<Order> {
        use crate::services::binance::filter::spot_market::base_quantity::{correct, filter};
        use crate::services::binance::math::is_within_price_ranges;
        use crate::services::binance::order::place_selling_market_order_with_base as place;

        if !is_within_price_ranges(price, &position.selling_price) {
            return None;
        }

        let base_quantity = correct(norms, price, &position.base_quantity).ok()?;
        filter(norms, price, &base_quantity).ok()?;

        let order = place(client, symbol, &base_quantity).await.ok()?;

        // Calculate the commission fee for the buy order and add it to the trade list
        let (trades, income_quote_quantity) = {
            let mut trades = Vec::with_capacity(3);
            let mut trades_all_quote_quantity = Decimal::ZERO;
            let mut trades_all_commission_quote_quantity = Decimal::ZERO;

            for fill in order.fills.iter() {
                let price = to_decimal(&fill.price).unwrap_or_default();
                let base_quantity = to_decimal(&fill.qty).unwrap_or_default();

                let commission = to_decimal(&fill.commission).unwrap_or_default();
                let quote_quantity_commission = commission;

                trades_all_quote_quantity += price * base_quantity;
                trades_all_commission_quote_quantity += commission;

                let trade = Trade {
                    price,
                    base_quantity,
                    quote_quantity_commission,
                };

                trades.push(trade)
            }

            let income_quote_quantity =
                trades_all_quote_quantity - trades_all_commission_quote_quantity;

            (trades, income_quote_quantity)
        };

        position.quote_quantity = position.quote_quantity + income_quote_quantity;
        position.base_quantity = position.base_quantity - base_quantity;

        return Some(Order {
            order_id: order.order_id,
            symbol: order.symbol,
            trades,
            side: order.side,
            timestamp: order.transact_time,
        });
    }

    fn to_decimal(value: &String) -> Option<Decimal> {
        use std::str::FromStr;

        Result::ok(Decimal::from_str(&value))
    }
}

pub mod buy {
    pub mod post_buy {
        pub const PATH: &str = "/binance/spot/order/buy";

        use binance::types::{OrderResponseFull, OrderSide, Symbol};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use crate::services::binance::client_with_sign;
        use crate::services::binance::types::Quantity;

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

            let result = client
                .spot_market_order_with_quote(
                    &p.symbol,
                    OrderSide::Buy,
                    &p.quote_quantity.to_string(),
                    None,
                )
                .await?;

            Ok(Response::ok(result))
        }
    }
}

pub mod sell {
    pub mod post_sell {
        pub const PATH: &str = "/binance/spot/order/sell";

        use binance::types::{OrderResponseFull, OrderSide, Symbol};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use crate::services::binance::client_with_sign;
        use crate::services::binance::types::Quantity;

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

            let result = client
                .spot_market_order_with_base(
                    &p.symbol,
                    OrderSide::Sell,
                    &p.base_quantity.to_string(),
                    None,
                )
                .await?;

            Ok(Response::ok(result))
        }
    }
}

pub mod info {
    pub mod post_info {
        pub const PATH: &str = "/binance/spot/order/info";

        use binance::types::{OrderInfo, Symbol, Trade};
        use serde::{Deserialize, Serialize};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Payload {
            api_key: String,
            secret_key: String,
            order_id: i64,
            symbol: Symbol,
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct Reply {
            order: OrderInfo,
            trades: Vec<Trade>,
        }

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<Reply> {
            let client = client_with_sign(p.api_key, p.secret_key)?;
            let order = client.spot_order_info(&p.symbol, p.order_id, None).await?;
            let trades = client.spot_trade(&p.symbol, p.order_id, None).await?;

            Ok(Response::ok(Reply { order, trades }))
        }
    }
}

pub mod trades {
    pub mod post_trades {
        pub const PATH: &str = "/binance/spot/order/trades";

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
