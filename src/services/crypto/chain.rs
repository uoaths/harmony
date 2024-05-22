use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlockChain {
    Ethereum = 1,
    Polygon  = 137,
}

mod block_chain_client {
    use std::sync::Arc;

    use crate::services::crypto::contract::Provider;

    use super::BlockChain;

    const NETWORK_GETWAY_ETHEREUM: &str = "https://eth.public-rpc.com";
    const NETWORK_GETWAY_POLYGON: &str = "https://polygon-rpc.com";

    impl BlockChain {
        #[rustfmt::skip]
        pub fn client(&self) -> Arc<Provider> {
            match self {
                Self::Ethereum => (*CLIENT_ETHEREUM).clone(),
                Self::Polygon  => (*CLIENT_POLYGON).clone(),
            }
        }
    }

    lazy_static::lazy_static! {
        pub static ref CLIENT_ETHEREUM: Arc<Provider> = Arc::new(Provider::try_from(NETWORK_GETWAY_ETHEREUM).expect("connect crypto provider error"));
    }

    lazy_static::lazy_static! {
        pub static ref CLIENT_POLYGON: Arc<Provider> = Arc::new( Provider::try_from(NETWORK_GETWAY_POLYGON).expect("connect crypto provider error"));
    }
}

mod block_chain_contract {
    use super::BlockChain;

    use super::super::contract::types::Address;
    use super::super::contract::{CaseERC20, CaseUniswapV3Factory, CaseUniswapV3Pool};
    use super::super::contract::{Contract, ContractResult};

    impl BlockChain {
        pub fn contract_erc_20(&self, address: Address) -> ContractResult<Contract<CaseERC20>> {
            let client = self.client();
            let contract_address = Contract::<()>::to_address(&address)?;
            let contract_case = CaseERC20::new(contract_address, client);

            Ok(Contract(address, contract_case))
        }

        pub fn contract_uniswap_v3_pool(
            &self,
            address: Address,
        ) -> ContractResult<Contract<CaseUniswapV3Pool>> {
            let client = self.client();
            let contract_address = Contract::<()>::to_address(&address)?;
            let contract_case = CaseUniswapV3Pool::new(contract_address, client);

            Ok(Contract(address, contract_case))
        }

        pub fn contract_uniswap_v3_factory(
            &self,
            address: Address,
        ) -> ContractResult<Contract<CaseUniswapV3Factory>> {
            let client = self.client();
            let contract_address = Contract::<()>::to_address(&address)?;
            let contract_case = CaseUniswapV3Factory::new(contract_address, client);

            Ok(Contract(address, contract_case))
        }
    }
}

mod block_chain_erc_20_tokens {
    use std::collections::HashMap;

    use super::super::contract::types::{Address, Symbol};
    use super::super::contract::{Contract, ContractError, ContractResult};
    use super::BlockChain;

    impl BlockChain {
        fn tokens(&self) -> &HashMap<&'static str, &'static str> {
            match self {
                Self::Ethereum => &*ETHERRUM_ERC_20_TOKENS,
                Self::Polygon => &*POLYGON_ERC_20_TOKENS,
            }
        }

        pub fn internal_erc_20(&self, symbol: &Symbol) -> ContractResult<Address> {
            let result = self
                .tokens()
                .get(symbol.as_str())
                .ok_or(ContractError::Address(format!(
                    "erc-20 token symbol {} not found",
                    symbol
                )))?
                .to_string();

            Ok(result)
        }

        pub fn lookup_or_validate_address(&self, str: String) -> ContractResult<Address> {
            if Contract::<()>::is_address(&str) {
                return Ok(str);
            };

            Ok(self.internal_erc_20(&str)?)
        }

        pub fn uniswap_v3_factory_address(&self) -> Address {
            match self {
                Self::Ethereum => String::from(ETHEREUM_UNISWAP_V3_FACTORY_ADDRESS),
                Self::Polygon => String::from(POLYGON_UNISWAP_V3_FACTORY_ADDRESS),
            }
        }
    }

    #[rustfmt::skip]
    lazy_static::lazy_static! {
        // ERC-20 token address in Polygon network
        // Tokens Symbol & Address Map
        // https://polygonscan.com/tokens
        static ref POLYGON_ERC_20_TOKENS: HashMap<&'static str, &'static str> = {
            let mut map = HashMap::new();
            map.insert("MATIC", "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"); // WMatic  18 decimals   Cross-Chain
            map.insert("USDC" , "0x2791bca1f2de4661ed88a30c99a7a9449aa84174"); // USDC.e  6  decimals   Cross-Chain
            map.insert("USDT" , "0xc2132d05d31c914a87c6611c10748aeb04b58e8f"); // USDT    6  decimals   Cross-Chain
            map.insert("WBTC" , "0x1bfd67037b42cf73acf2047067bd4f2c47d9bfd6"); // WBTC    8  decimals
            map.insert("WETH" , "0x7ceb23fd6bc0add59e62ac25578270cff1b9f619"); // WETH    18 decimals   Cross-Chain
            map.insert("BNB"  , "0x3BA4c387f786bFEE076A58914F5Bd38d668B42c3"); // BNB     18 decimals
            map.insert("UNI"  , "0xb33eaad8d922b1083446dc23f610c2567fb5180f"); // UNI     18 decimals

            map
        };
    }

    #[rustfmt::skip]
    lazy_static::lazy_static! {
        // ERC-20 token address in Ethereum network
        // Tokens Symbol & Address Map
        static ref ETHERRUM_ERC_20_TOKENS: HashMap<&'static str, &'static str> = {
            let mut map = HashMap::new();
            map.insert("USDC", "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"); // USDC    6  decimals
            map.insert("USDT", "0xdac17f958d2ee523a2206206994597c13d831ec7"); // USDT    6  decimals
            map.insert("WBTC", "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599"); // WBTC    8  decimals
            map.insert("WETH", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"); // WETH    18 decimals
            map.insert("BNB" , "0xB8c77482e45F1F44dE1745F52C74426C631bDD52"); // BNB     18 decimals
            map.insert("UNI" , "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984"); // UNI     18 decimals

            map
        };
    }

    pub const POLYGON_UNISWAP_V3_FACTORY_ADDRESS: &str =
        "0x1F98431c8aD98523631AE4a59f267346ea31F984";
    pub const ETHEREUM_UNISWAP_V3_FACTORY_ADDRESS: &str =
        "0x1F98431c8aD98523631AE4a59f267346ea31F984";
}
