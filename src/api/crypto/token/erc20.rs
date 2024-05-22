pub mod get {
    pub const PATH: &str = "/crypto/token/erc20";

    #[cfg(feature = "server-api-handler")]
    pub mod handler {
        use crate::api::http::request::Query;
        use crate::api::http::response::{Response, ResponseResult};
        use crate::api::http::trip::Trip;

        use super::models::{Params, ResponseBody};

        #[tracing::instrument(skip(_c))]
        pub async fn handler(_c: Trip, Query(p): Query<Params>) -> ResponseResult<ResponseBody> {
            let address = p.chain.lookup_or_validate_address(p.token)?;

            let token = p.chain.contract_erc_20(address.clone())?;

            let result = ResponseBody {
                address,
                name: token.name().await?,
                symbol: token.symbol().await?,
                decimals: token.decimals().await?,
                total_supply: token.total_supply().await?,
            };

            Ok(Response::ok(result))
        }
    }

    #[cfg(feature = "server-api-models")]
    pub mod models {
        use serde::{Deserialize, Serialize};

        use crate::services::crypto::chain::BlockChain;
        use crate::services::crypto::contract::types::{Address, Symbol, Uint256, Uint8};

        #[rustfmt::skip]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Params {
            pub chain:   BlockChain,
            pub token:   String,
        }

        #[rustfmt::skip]
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ResponseBody {
            pub name:         String,
            pub symbol:       Symbol,
            pub address:      Address,
            pub decimals:     Uint8,
            pub total_supply: Uint256,
        }
    }
}
