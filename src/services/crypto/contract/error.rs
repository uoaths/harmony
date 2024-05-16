#[derive(Debug)]
pub enum ContractError {
    Address(String),
    Execution(String),
    Calculation(String),
}

mod contract_error {
    use std::error::Error;
    use std::fmt::{Display, Formatter, Result};

    use bigdecimal::ParseBigDecimalError;

    use super::ContractError;

    impl Error for ContractError {}
    impl Display for ContractError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self.to_string())
        }
    }

    impl From<ParseBigDecimalError> for ContractError {
        fn from(err: ParseBigDecimalError) -> Self {
            Self::Calculation(err.to_string())
        }
    }
}
