pub mod math;

use std::{error::Error, time::Duration};

use binance::prelude::*;

pub fn client() -> Result<Client, Box<dyn Error>> {
    let result = ClientBuilder::new().build()?;

    Ok(result)
}

pub fn client_with_sign(api_key: String, secret_key: String) -> Result<Client, Box<dyn Error>> {
    let result = ClientBuilder::new()
        .set_api_key(api_key)
        .set_secret_key(secret_key)
        .set_timeout(Duration::from_secs(5))
        .build()?;

    Ok(result)
}
