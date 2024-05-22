pub mod get {
    pub const PATH: &str = "/crypto/token/price";

    #[cfg(feature = "server-api-handler")]
    pub mod handler {
        use crate::api::http::request::Query;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use super::models::{Params, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Query(p): Query<Params>) -> ResponseResult<ResponseBody> {
            // Factory Contract
            let factory = p.chain.uniswap_v3_factory_address();
            let factory = p.chain.contract_uniswap_v3_factory(factory)?;

            let base = p.chain.lookup_or_validate_address(p.base)?;
            let quote = p.chain.lookup_or_validate_address(p.quote)?;

            // Pool Contract
            let pool = factory.get_pool(&base, &quote, p.fee).await?;
            let pool = p.chain.contract_uniswap_v3_pool(pool)?;

            let result = ResponseBody {
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

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use serde::{Deserialize, Serialize};

        use crate::services::crypto::chain::BlockChain;
        use crate::services::crypto::contract::types::{Uint24, Uint256};

        #[rustfmt::skip]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Params {
            pub chain: BlockChain,
            pub base:  String, // Address or Symbol
            pub quote: String,
            pub fee:   Uint24,
        }

        #[rustfmt::skip]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub price: Uint256,
        }
    }
}
