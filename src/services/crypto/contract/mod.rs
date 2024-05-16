mod abi;
mod erc20;
mod error;
mod uniswap;

/// Contract Struct
pub struct Contract<T>(pub types::Address, pub T);

/// Contract Case
pub use abi::{CaseERC20, CaseUniswapV3Factory, CaseUniswapV3Pool, Provider};

/// Contract Error
pub use error::ContractError;

pub type ContractResult<T> = Result<T, ContractError>;

#[rustfmt::skip]
pub mod types {
    pub type Symbol  = String;
    pub type Address = String;
    pub type Uint8   = u8;
    pub type Uint16  = u16;
    pub type Uint24  = u32;
    pub type Uint128 = u128;
    pub type Uint160 = String;
    pub type Uint256 = String;
    pub type Int24   = i32;
    pub type Bool    = bool;
}
