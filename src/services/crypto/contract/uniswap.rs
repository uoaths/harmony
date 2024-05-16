mod custom_uniswap_v3_math {
    const NUM_2_POW_192: &str = "6277101735386680763835789423207666416102355444464034512896";

    use std::str::FromStr;

    use bigdecimal::{num_bigint::BigInt, BigDecimal};

    use super::super::types::*;
    use super::super::ContractResult;

    pub fn sqrt_price_x96_to_price(value: &Uint256, decimals: u32) -> ContractResult<Uint256> {
        let value = BigDecimal::from_str(value)?;
        let num = BigDecimal::from_str(NUM_2_POW_192)?;
        let result = (value.square() / num) / num_10_pow_of(decimals);

        Ok(result.to_string())
    }

    pub fn sqrt_price_x96_to_price_inverse(
        value: &Uint256,
        decimals: u32,
    ) -> ContractResult<Uint256> {
        let value = BigDecimal::from_str(value)?;
        let num = BigDecimal::from_str(NUM_2_POW_192)?;

        let result = num_10_pow_of(decimals) / (value.square() / num);

        Ok(result.to_string())
    }

    pub fn num_10_pow_of(exponent: u32) -> BigDecimal {
        BigDecimal::new(BigInt::from(10).pow(exponent), 0)
    }
}

mod contract_uniswap_v3_pool {
    use serde::Serialize;

    use super::super::abi::{CaseERC20, CaseUniswapV3Pool};
    use super::super::types::*;
    use super::super::{Contract, ContractResult};

    /// Uniswap V3 Pool Custom Contract Functions
    impl Contract<CaseUniswapV3Pool> {
        pub async fn price(
            &self,
            base: &Contract<CaseERC20>,
            quote: &Contract<CaseERC20>,
        ) -> ContractResult<Uint256> {
            use super::custom_uniswap_v3_math::{
                sqrt_price_x96_to_price, sqrt_price_x96_to_price_inverse,
            };

            let slot_0 = self.slot_0().await?;

            // ERC-20 Token decimals abs
            let decimals =
                (base.decimals().await? as i16 - quote.decimals().await? as i16).abs() as u32;

            if base.is_address_match(&self.token_0().await?) {
                sqrt_price_x96_to_price(&slot_0.sqrt_price_x96, decimals)
            } else {
                sqrt_price_x96_to_price_inverse(&slot_0.sqrt_price_x96, decimals)
            }
        }
    }

    #[rustfmt::skip]
    #[derive(Serialize)]
    pub struct Slot0 {
        pub sqrt_price_x96:               Uint160,
        pub tick:                         Int24,
        pub observation_index:            Uint16,
        pub observation_cardinality:      Uint16,
        pub observation_cardinality_next: Uint16,
        pub fee_protocol:                 Uint8,
        pub unlocked:                     Bool,
    }

    /// Uniswap V3 Pool Read Contract Functions
    impl Contract<CaseUniswapV3Pool> {
        pub async fn slot_0(&self) -> ContractResult<Slot0> {
            let execute = Self::contract_call(self.1.slot_0().call().await)?;

            let (
                sqrt_price_x96,
                tick,
                observation_index,
                observation_cardinality,
                observation_cardinality_next,
                fee_protocol,
                unlocked,
            ) = execute;

            let result = Slot0 {
                sqrt_price_x96: sqrt_price_x96.to_string(),
                tick,
                observation_index,
                observation_cardinality,
                observation_cardinality_next,
                fee_protocol,
                unlocked,
            };

            Ok(result)
        }

        pub async fn fee(&self) -> ContractResult<Uint24> {
            let result = Self::contract_call(self.case().fee().call().await)?;

            Ok(result)
        }

        pub async fn fee_growth_global_0x128(&self) -> ContractResult<Uint256> {
            let result = Self::contract_call(self.case().fee_growth_global_0x128().call().await)?;

            Ok(result.to_string())
        }

        pub async fn fee_growth_global_1x128(&self) -> ContractResult<Uint256> {
            let result = Self::contract_call(self.case().fee_growth_global_1x128().call().await)?;

            Ok(result.to_string())
        }

        pub async fn liquidity(&self) -> ContractResult<Uint128> {
            let result = Self::contract_call(self.case().liquidity().call().await)?;

            Ok(result)
        }

        pub async fn factory(&self) -> ContractResult<Address> {
            use ethers::types::H160;
            use ethers::utils::hex::ToHexExt;

            let result: H160 = Self::contract_call(self.case().factory().call().await)?;

            Ok(result.encode_hex_with_prefix())
        }

        pub async fn token_0(&self) -> ContractResult<Address> {
            use ethers::types::H160;
            use ethers::utils::hex::ToHexExt;

            let result: H160 = Self::contract_call(self.case().token_0().call().await)?;

            Ok(result.encode_hex_with_prefix())
        }

        pub async fn token_1(&self) -> ContractResult<Address> {
            use ethers::types::H160;
            use ethers::utils::hex::ToHexExt;

            let result: H160 = Self::contract_call(self.case().token_1().call().await)?;

            Ok(result.encode_hex_with_prefix())
        }
    }
}

mod contract_uniswap_v3_factory {
    use super::super::abi::CaseUniswapV3Factory;
    use super::super::types::*;
    use super::super::{Contract, ContractResult};

    impl Contract<CaseUniswapV3Factory> {
        pub async fn get_pool(
            &self,
            address_0: &Address,
            address_1: &Address,
            fee: Uint24,
        ) -> ContractResult<Address> {
            use ethers::types::H160;
            use ethers::utils::hex::ToHexExt;

            // FIX: stack overflow when params not true
            let address_0 = Self::to_address(address_0)?;
            let address_1 = Self::to_address(address_1)?;

            let result: H160 =
                Self::contract_call(self.1.get_pool(address_0, address_1, fee).call().await)?;

            Ok(result.encode_hex_with_prefix())
        }
    }
}
