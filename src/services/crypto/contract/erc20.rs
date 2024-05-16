use super::abi::CaseERC20;
use super::{types::*, Contract, ContractResult};

// Read Contract ERC-20
impl Contract<CaseERC20> {
    pub async fn name(&self) -> ContractResult<String> {
        let result = Self::contract_call(self.1.name().call().await)?;

        Ok(result)
    }

    pub async fn symbol(&self) -> ContractResult<Symbol> {
        let result = Self::contract_call(self.1.symbol().call().await)?;

        Ok(result)
    }

    pub async fn decimals(&self) -> ContractResult<Uint8> {
        let result = Self::contract_call(self.1.decimals().call().await)?;

        Ok(result)
    }

    pub async fn total_supply(&self) -> ContractResult<Uint256> {
        use ethers::types::U256;

        let result: U256 = Self::contract_call(self.1.total_supply().call().await)?;

        Ok(result.to_string())
    }
}
