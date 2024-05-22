#[cfg(feature = "server-api-handler")]
mod http;

#[cfg(feature = "server")]
pub use self::http::trip::State;

pub mod general;

#[cfg(feature = "service-crypto")]
pub mod crypto;

#[cfg(feature = "service-binance")]
pub mod binance;
