#[derive(Debug)]
pub enum SymbolFilterError {
    Decimal(String),
    Precision(String),
    LotSize(String),
    Notional(String),
    MinNotional(String),
    MarketLotSize(String),
}

impl std::error::Error for SymbolFilterError {}
impl std::fmt::Display for SymbolFilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Decimal(e) => e.to_string(),
            Self::LotSize(e) => format!("LOTSIZE {}", e),
            Self::Notional(e) => format!("NOTIONAL {}", e),
            Self::Precision(e) => format!("PRECISION {}", e),
            Self::MinNotional(e) => format!("MIN_NOTIONAL {}", e),
            Self::MarketLotSize(e) => format!("MARKET_LOT_SIZE {}", e),
        };

        write!(f, "FILTER {}", message)
    }
}

impl From<plot::error::Error> for SymbolFilterError {
    fn from(value: plot::error::Error) -> Self {
        Self::Decimal(value.to_string())
    }
}
