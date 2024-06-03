pub mod filter;

use binance::{
    prelude::{Client, ClientBuilder},
    types::{OrderSide, SymbolInfo},
};
use filter::error::SymbolFilterError;
use plot::{
    trade::{Trade, Trader},
    types::{BaseQuantity, Decimal, Price, QuoteQuantity},
};
use std::{error::Error, time::Duration};

pub fn client() -> Result<Client, Box<dyn Error>> {
    let result = ClientBuilder::new().build()?;

    Ok(result)
}

pub fn client_with_sign(api_key: String, secret_key: String) -> Result<Client, Box<dyn Error>> {
    let result = ClientBuilder::new()
        .set_api_key(api_key)
        .set_secret_key(secret_key)
        .set_timeout(Duration::from_secs(5))
        .build()?;

    Ok(result)
}

pub struct BinanceSpotTest {
    normal: SymbolInfo,
    commission: Decimal,
}

impl BinanceSpotTest {
    pub fn new(normal: SymbolInfo, commission: Decimal) -> Self {
        Self { normal, commission }
    }
}

impl Trader for BinanceSpotTest {
    async fn buy(
        &self,
        price: &Price,
        quantity: &QuoteQuantity,
    ) -> Result<Vec<Trade>, Box<dyn Error>> {
        let quote_quantity = self.normal.correct_quote_quantity(price, quantity)?;
        self.normal.filter_quote_quantity(price, &quote_quantity)?;
        let base_quantity = (quote_quantity / price) * (Decimal::ONE - self.commission);
        let base_quantity = base_quantity.trunc_with_scale(self.normal.base_asset_precision.into());
        let trade = Trade::with_buy(price.clone(), base_quantity, quote_quantity);

        Ok(vec![trade])
    }

    async fn sell(
        &self,
        price: &Price,
        quantity: &BaseQuantity,
    ) -> Result<Vec<Trade>, Box<dyn Error>> {
        let base_quantity = self.normal.correct_base_quantity(price, quantity)?;
        self.normal.filter_base_quantity(price, &base_quantity)?;
        let quote_quantity = (base_quantity * price) * (Decimal::ONE - self.commission);
        let quote_quantity =
            quote_quantity.trunc_with_scale(self.normal.quote_asset_precision.into());
        let trade = Trade::with_sell(price.clone(), base_quantity, quote_quantity);

        Ok(vec![trade])
    }
}

pub struct BinanceSpot {
    client: Client,
    normal: SymbolInfo,
}

impl BinanceSpot {
    pub fn new(normal: SymbolInfo, client: Client) -> Self {
        Self { client, normal }
    }
}

impl Trader for BinanceSpot {
    async fn buy(
        &self,
        price: &Price,
        quantity: &QuoteQuantity,
    ) -> Result<Vec<Trade>, Box<dyn Error>> {
        let quantity = self.normal.correct_quote_quantity(price, quantity)?;
        self.normal.filter_quote_quantity(price, &quantity)?;

        // Buy the base quantity by the quoted quantity
        let order = self
            .client
            .spot_market_order_with_quote(
                &self.normal.symbol,
                OrderSide::Buy,
                &quantity.to_string(),
                None,
            )
            .await?;

        Ok(order.to_trades())
    }

    async fn sell(
        &self,
        price: &Price,
        quantity: &BaseQuantity,
    ) -> Result<Vec<Trade>, Box<dyn Error>> {
        let quantity = self.normal.correct_base_quantity(price, quantity)?;
        self.normal.filter_base_quantity(price, &quantity)?;

        let order = self
            .client
            .spot_market_order_with_base(
                &self.normal.symbol,
                OrderSide::Sell,
                &quantity.to_string(),
                None,
            )
            .await?;

        Ok(order.to_trades())
    }
}

pub trait ConvertTrades {
    fn to_trades(&self) -> Vec<Trade>;
}

pub trait ConvertFilter {
    fn filter_base_quantity(
        &self,
        price: &Price,
        quantity: &BaseQuantity,
    ) -> Result<(), SymbolFilterError>;
    fn filter_quote_quantity(
        &self,
        price: &Price,
        quantity: &QuoteQuantity,
    ) -> Result<(), SymbolFilterError>;
    fn correct_base_quantity(
        &self,
        price: &Price,
        quantity: &BaseQuantity,
    ) -> Result<BaseQuantity, SymbolFilterError>;
    fn correct_quote_quantity(
        &self,
        price: &Price,
        quantity: &QuoteQuantity,
    ) -> Result<QuoteQuantity, SymbolFilterError>;
}

mod order_extend {
    use super::ConvertTrades;
    use binance::types::{OrderResponseFull, OrderSide};
    use plot::trade::Trade;
    use plot::types::Decimal;

    impl ConvertTrades for OrderResponseFull {
        fn to_trades(&self) -> Vec<Trade> {
            let mut trades = Vec::with_capacity(2);
            match self.side {
                OrderSide::Buy => {
                    for i in self.fills.iter() {
                        let price = dec(&i.price);
                        let base_quantity = dec(&i.qty);
                        let base_quantity_commission = dec(&i.commission);
                        let quote_quantity = price * base_quantity;
                        let base_quantity = base_quantity - base_quantity_commission;

                        trades.push(Trade::with_buy(price, base_quantity, quote_quantity))
                    }
                }
                OrderSide::Sell => {
                    for i in self.fills.iter() {
                        let price = dec(&i.price);
                        let base_quantity = dec(&i.qty);
                        let quote_quantity_commission = dec(&i.commission);
                        let quote_quantity = (price * base_quantity) - quote_quantity_commission;

                        trades.push(Trade::with_sell(price, base_quantity, quote_quantity))
                    }
                }
            };

            trades
        }
    }

    fn dec(value: &String) -> Decimal {
        use std::str::FromStr;

        Decimal::from_str(&value).unwrap_or_default()
    }
}
