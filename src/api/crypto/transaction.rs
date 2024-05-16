pub(super) mod get_gas {
    pub const PATH: &str = "/crypto/gas";

    use ethers::providers::Middleware;
    use serde::{Deserialize, Serialize};

    use crate::api::http::request::Query;
    use crate::api::http::response::Response;
    use crate::api::http::trip::Trip;

    use super::super::BlockChain;

    #[derive(Debug, Deserialize)]
    pub struct Params {
        chain: BlockChain,
    }

    #[derive(Serialize)]
    pub struct Reply {
        price: String,
        timestamp: u128,
    }

    #[tracing::instrument(skip(c))]
    pub async fn handler(c: Trip, Query(p): Query<Params>) -> Response<Reply> {
        let price = p.chain.client().get_gas_price().await.unwrap();

        Response::ok(Reply {
            price: price.to_string(),
            timestamp: c.timestamp_millis(),
        })
    }
}

// pub(super) mod post_transaction {
//     pub const PATH: &str = "/crypto/transaction";

//     use ethers::core::types::Address;
//     use ethers::utils::hex::ToHexExt;
//     use serde::{Deserialize, Serialize};

//     use crate::api::http::request::Query;
//     use crate::api::http::response::Response;
//     use crate::api::http::trip::Trip;

//     #[derive(Debug, Deserialize)]
//     pub struct Params {
//         address: String,
//     }

//     #[derive(Serialize)]
//     pub struct Reply {
//     }

//     #[tracing::instrument(skip(c))]
//     pub async fn handler(c: Trip, Query(p): Query<Params>) -> Response<Reply> {
//         let contract_address = p.address.parse::<Address>().unwrap();
//         c.crypto_client().t
//         let pool = super::IUniswapV3Pool::new(contract_address, c.crypto_client().into());

//         Response::ok(Reply {})
//     }
// }

// use std::{
//     error::Error,
//     ops::{Div, Mul},
//     sync::Arc,
// };

// use ethers::{
//     contract::abigen,
//     core::{
//         types::{Address, I256, U256},
//         utils::format_units,
//     },
//     providers::{Http, Middleware, Provider},
// };

// abigen!(
//     AggregatorInterface,
//     r#"[
//         latestAnswer() public view virtual override returns (int256 answer)
//     ]"#,
// );

// const ETH_DECIMALS: u32 = 18;
// const USD_PRICE_DECIMALS: u32 = 8;
// const ETH_USD_FEED: &str = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419";
// const RPC_URI: &str = "https://eth.llamarpc.com";

// /// Retrieves the USD amount per gas unit, using a Chainlink price oracle.
// /// Function gets the amount of `wei` to be spent per gas unit then multiplies
// /// for the ETH USD value.
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let client = get_client();
//     let oracle = get_oracle(&client);

//     let usd_per_eth: I256 = oracle.latest_answer().call().await?;
//     let usd_per_eth: U256 = U256::from(usd_per_eth.as_u128());
//     let wei_per_gas: U256 = client.get_gas_price().await?;

//     // Gas stations use to report gas price in gwei units (1 gwei = 10^9 wei)
//     let gwei: f64 = format_units(wei_per_gas, "gwei")?.parse::<f64>()?;

//     // Let's convert the gas price to USD
//     let usd_per_gas: f64 = usd_value(wei_per_gas, usd_per_eth)?;

//     println!(
//         r#"
//         Gas price
//         ---------------
//         {gwei:>10.2} gwei
//         {usd_per_gas:>10.8} usd
//         "#
//     );
//     Ok(())
// }

// /// `amount`: Number of wei per gas unit (18 decimals)
// /// `price_usd`: USD price per ETH (8 decimals)
// fn usd_value(amount: U256, price_usd: U256) -> Result<f64, Box<dyn Error>> {
//     let base: U256 = U256::from(10).pow(ETH_DECIMALS.into());
//     let value: U256 = amount.mul(price_usd).div(base);
//     let f: String = format_units(value, USD_PRICE_DECIMALS)?;
//     Ok(f.parse::<f64>()?)
// }

// fn get_client() -> Arc<Provider<Http>> {
//     let provider: Provider<Http> = Provider::<Http>::try_from(RPC_URI).expect("Valid URL");
//     Arc::new(provider)
// }

// fn get_oracle(client: &Arc<Provider<Http>>) -> AggregatorInterface<Provider<Http>> {
//     let address: Address = ETH_USD_FEED.parse().expect("Valid address");
//     AggregatorInterface::new(address, Arc::clone(client))
// }
