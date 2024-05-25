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
        use ploy::types::{Decimal, Price};

        use crate::api::http::request::Json;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;
        use crate::services::binance::client_with_sign;

        use super::models::{Order, Payload, Position, ResponseBody, Trade};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Json(p): Json<Payload>) -> ResponseResult<ResponseBody> {
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

            if !is_within_ranges(price, &position.buying_price) {
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
            use crate::services::binance::filter::spot::base_quantity::{correct, filter};
            use crate::services::binance::order::place_selling_market_order_with_base as place;

            if !is_within_ranges(price, &position.selling_price) {
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

    pub mod models {
        use binance::types::{OrderSide, Symbol};
        use ploy::{
            math::Range,
            types::{Price, Quantity},
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
            pub side: OrderSide,
            pub timestamp: u128,
            pub trades: Vec<Trade>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Trade {
            pub price: Price,
            pub base_quantity: Quantity,
            pub quote_quantity_commission: Quantity,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Position {
            pub buying_price: Vec<Range<Price>>,
            pub selling_price: Vec<Range<Price>>,
            pub base_quantity: Quantity,
            pub quote_quantity: Quantity,
        }
    }
}
