pub mod get {
    pub const PATH: &str = "/crypto/uniswap";

    pub mod handler {
        use crate::api::http::request::Query;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use super::models::{Params, ResponseBody, Slot0};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Query(p): Query<Params>) -> ResponseResult<ResponseBody> {
            let factory_contract_address = p.chain.uniswap_v3_factory_address();
            let factory_contract = p
                .chain
                .contract_uniswap_v3_factory(factory_contract_address)?;

            let token_0 = p.chain.lookup_or_validate_address(p.token_0)?;
            let token_1 = p.chain.lookup_or_validate_address(p.token_1)?;

            // get pool address by factory contract
            let pool_contract_address =
                factory_contract.get_pool(&token_0, &token_1, p.fee).await?;

            let pool_contract = p
                .chain
                .contract_uniswap_v3_pool(pool_contract_address.clone())?;

            let slot_0 = pool_contract.slot_0().await?;

            let slot_0 = Slot0 {
                sqrt_price_x96: slot_0.sqrt_price_x96,
                tick: slot_0.tick,
                observation_cardinality: slot_0.observation_cardinality,
                observation_cardinality_next: slot_0.observation_cardinality_next,
                observation_index: slot_0.observation_index,
                fee_protocol: slot_0.fee_protocol,
                unlocked: slot_0.unlocked,
            };

            let result = Response::ok(ResponseBody {
                address: pool_contract_address,
                slot_0,
                factory: pool_contract.factory().await?,
                fee: pool_contract.fee().await?,
                fee_growth_global_0x128: pool_contract.fee_growth_global_0x128().await?,
                fee_growth_global_1x128: pool_contract.fee_growth_global_1x128().await?,
                liquidity: pool_contract.liquidity().await?,
                token_0: pool_contract.token_0().await?,
                token_1: pool_contract.token_1().await?,
            });

            Ok(result)
        }
    }

    pub mod models {
        use serde::{Deserialize, Serialize};

        use crate::services::crypto::chain::BlockChain;
        use crate::services::crypto::contract::types::*;

        #[rustfmt::skip]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Params {
            pub chain:   BlockChain,
            pub fee:     Uint24,
            pub token_0: String,  // Address or Symbol
            pub token_1: String,  // Address or Symbol
        }

        #[rustfmt::skip]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Slot0 {
            pub sqrt_price_x96:               Uint160,
            pub tick:                         Int24,
            pub observation_index:            Uint16,
            pub observation_cardinality:      Uint16,
            pub observation_cardinality_next: Uint16,
            pub fee_protocol:                 Uint8,
            pub unlocked:                     Bool,
        }

        #[rustfmt::skip]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            // Uniswap Pool Address
            pub address:                  Address,
            pub factory:                  Address,
            pub fee:                      Uint24,
            pub liquidity:                Uint128,
            pub fee_growth_global_0x128:  Uint256,
            pub fee_growth_global_1x128:  Uint256,
            pub token_0:                  Address,
            pub token_1:                  Address,
            pub slot_0:                   Slot0,
        }
    }
}
