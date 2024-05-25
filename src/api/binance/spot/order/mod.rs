pub mod buy;
pub mod info;
pub mod sell;
pub mod trades;

pub mod post {
    pub const PATH: &str = "/binance/spot/order";

    pub mod handler {
        use binance::prelude::Client;
        use binance::types::{Symbol, SymbolInfo};
        use ploy::math::is_within_ranges;
        use ploy::position::{Position, Trade};
        use ploy::types::{Decimal, Price};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        use super::models::{Order, Payload, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<ResponseBody> {
            let client = client_with_sign(p.api_key, p.secret_key)?;
            let norm = client.exchange_info(&p.symbol).await?;
            let price = client.price(&p.symbol).await?.price;
            let price = dec(&price);

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

            Ok(Response::ok(ResponseBody {
                positions,
                order,
                price,
                symbol: p.symbol,
            }))
        }

        async fn buy(
            client: &Client,
            symbol: &Symbol,
            price: &Price,
            norms: &SymbolInfo,
            position: &mut Position,
        ) -> Option<Order> {
            use crate::services::binance::filter::spot::quote_quantity::{correct, filter};
            use crate::services::binance::order::place_buying_market_order_with_quote as place;

            if !is_within_ranges(price, &position.buying_prices) {
                return None;
            }

            // Filter the number of quotes to be bought.
            // If the filter is not successfully passed, None will be returned.
            let quote_quantity = correct(norms, price, &position.quote_quantity).ok()?;
            filter(norms, price, &quote_quantity).ok()?;

            // Buy the base quantity by the quoted quantity
            let order = place(client, symbol, &quote_quantity).await.ok()?;

            // Calculate the commission fee for the buy order and add it to the trade list
            let trades = {
                let mut trades = Vec::with_capacity(3);

                for fill in order.fills.iter() {
                    // fills fixedly returns the base quantity in the qty field
                    let price = dec(&fill.price);
                    let base_quantity = dec(&fill.qty);
                    let base_quantity_commission = dec(&fill.commission);
                    let quote_quantity = price * base_quantity;
                    position.base_quantity += (base_quantity - base_quantity_commission);
                    position.quote_quantity -= quote_quantity;

                    trades.push(Trade::with_buy_side(price, base_quantity, quote_quantity))
                }

                trades
            };

            return Some(Order {
                order_id: order.order_id,
                symbol: order.symbol,
                trades,
            });
        }

        async fn sell(
            client: &Client,
            symbol: &Symbol,
            price: &Price,
            norms: &SymbolInfo,
            position: &mut Position,
        ) -> Option<Order> {
            use crate::services::binance::filter::spot::base_quantity::{correct, filter};
            use crate::services::binance::order::place_selling_market_order_with_base as place;

            if !is_within_ranges(price, &position.selling_prices) {
                return None;
            }

            let base_quantity = correct(norms, price, &position.base_quantity).ok()?;
            filter(norms, price, &base_quantity).ok()?;

            let order = place(client, symbol, &base_quantity).await.ok()?;

            // Calculate the commission fee for the buy order and add it to the trade list
            let trades = {
                let mut trades = Vec::with_capacity(3);

                for fill in order.fills.iter() {
                    let price = dec(&fill.price);
                    let base_quantity = dec(&fill.qty);
                    let quote_quantity_commission = dec(&fill.commission);
                    let quote_quantity = (price * base_quantity) - quote_quantity_commission;
                    position.base_quantity -= base_quantity;
                    position.quote_quantity += quote_quantity;

                    trades.push(Trade::with_sell_side(price, base_quantity, quote_quantity))
                }

                trades
            };

            return Some(Order {
                order_id: order.order_id,
                symbol: order.symbol,
                trades,
            });
        }

        fn dec(value: &String) -> Decimal {
            use std::str::FromStr;

            Decimal::from_str(&value).unwrap_or_default()
        }
    }

    pub mod models {
        use binance::types::Symbol;
        use ploy::{
            position::{Position, Trade},
            types::Price,
        };
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Payload {
            pub api_key: String,
            pub secret_key: String,
            pub symbol: Symbol,
            pub positions: Vec<Position>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub symbol: Symbol,
            pub price: Price,
            pub positions: Vec<Position>,
            pub order: Vec<Order>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Order {
            pub order_id: i64,
            pub symbol: Symbol,
            pub trades: Vec<Trade>,
        }
    }
}
