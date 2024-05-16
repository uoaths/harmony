pub(super) mod get_token_erc20 {
    pub const PATH: &str = "/crypto/token/erc20";

    use serde::{Deserialize, Serialize};

    use crate::api::http::request::Query;
    use crate::api::http::response::{Response, ResponseResult};
    use crate::api::http::trip::Trip;
    use crate::services::crypto::chain::BlockChain;
    use crate::services::crypto::contract::types::{Address, Symbol, Uint256, Uint8};

    #[rustfmt::skip]
    #[derive(Debug, Deserialize)]
    pub struct Params {
        chain:   BlockChain,
        token:   String,
    }

    #[rustfmt::skip]
    #[derive(Serialize)]
    pub struct Reply {
        name:         String,
        symbol:       Symbol,
        address:      Address,
        decimals:     Uint8,
        total_supply: Uint256,
    }

    #[tracing::instrument(skip(_c))]
    pub async fn handler(_c: Trip, Query(p): Query<Params>) -> ResponseResult<Reply> {
        let address = p.chain.lookup_or_validate_address(p.token)?;

        let token = p.chain.contract_erc_20(address.clone())?;

        let result = Reply {
            address,
            name: token.name().await?,
            symbol: token.symbol().await?,
            decimals: token.decimals().await?,
            total_supply: token.total_supply().await?,
        };

        Ok(Response::ok(result))
    }
}

pub(super) mod get_token_price {
    pub const PATH: &str = "/crypto/token/price";

    use serde::{Deserialize, Serialize};

    use crate::api::http::request::Query;
    use crate::api::http::response::{Response, ResponseResult};
    use crate::api::http::trip::Trip;
    use crate::services::crypto::chain::BlockChain;
    use crate::services::crypto::contract::types::{Uint24, Uint256};

    #[rustfmt::skip]
    #[derive(Debug, Deserialize)]
    pub struct Params {
        chain: BlockChain,
        base:  String, // Address or Symbol
        quote: String,
        fee:   Uint24,
    }

    #[rustfmt::skip]
    #[derive(Serialize)]
    pub struct Reply {
        price: Uint256,
    }

    #[tracing::instrument(skip(_c))]
    pub async fn handler(_c: Trip, Query(p): Query<Params>) -> ResponseResult<Reply> {
        // Factory Contract
        let factory = p.chain.uniswap_v3_factory_address();
        let factory = p.chain.contract_uniswap_v3_factory(factory)?;

        let base = p.chain.lookup_or_validate_address(p.base)?;
        let quote = p.chain.lookup_or_validate_address(p.quote)?;

        // Pool Contract
        let pool = factory.get_pool(&base, &quote, p.fee).await?;
        let pool = p.chain.contract_uniswap_v3_pool(pool)?;

        let result = Reply {
            price: pool
                .price(
                    &p.chain.contract_erc_20(base)?,
                    &p.chain.contract_erc_20(quote)?,
                )
                .await?,
        };

        Ok(Response::ok(result))
    }
}
