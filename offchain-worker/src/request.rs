use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use sp_core::H160;
use std::str::FromStr;

#[derive(Debug)]
pub enum Error {
	ReqwestError(reqwest::Error),
	CustomError(String),
}

impl From<reqwest::Error> for Error {
	fn from(value: reqwest::Error) -> Self {
		Error::ReqwestError(value)
	}
}

pub async fn fetch_jur_balance(addr: &H160) -> Result<u128, Error> {
	let hex_encoded_addr = hex::encode(addr.0);
	let request_address =
		format!("http://localhost:3000/balance?account=0x{}", hex_encoded_addr.clone());
	let response = reqwest::get(request_address).await?;
	let status = response.status().as_u16();
	if status != 200 {
		log::warn!("Unexpected status code: {}", status);
	}
	let balance_str = response.text().await?;
	// TODO: chain decimals are hardcoded; what shall we do about this? get from client maybe?
	let balance = BigUint::from_str(&balance_str).map_err(|e| {
		log::error!(target: "offchain-worker-service", "Failed to parse balance to BigUint: {:?}", e);
		Error::CustomError(String::from("Failed to parse balance from str to BigUint"))
	})?;

	let Some(balance) = balance.to_u128() else {
		log::error!(target: "offchain-worker-service", "Failed to convert BigUint to u128");
		return Err(Error::CustomError(String::from("Failed to convert BigUint to u128")))
	};
	Ok(balance)
}
