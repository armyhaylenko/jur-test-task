// Expose a public function for retrieving the JSON object with all configuration and public key

use super::key_mgmt::{get_public_key, KeyError};

use core::time::Duration;
use primitives::shared::{Public, PUBLIC_KEY_TYPE_ID};
use sc_keystore::LocalKeystore;

use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub enum ConfigError {
	/// Unable to convert public key to json string
	JSONConversionError,
	/// Error related to Keystore
	KeystoreError(KeyError),
}

/// Returns KeyInfo once available else asynchronously waits for it.
pub async fn get_key_info(keystore: &Arc<LocalKeystore>) -> Public {
	loop {
		if let Ok(key_info) = get_public_key(PUBLIC_KEY_TYPE_ID, keystore).await {
			return key_info
		}
		log::info!(target: "runtime::runtime-plugin", "Asynchronously waiting for public key");
		tokio::time::sleep(Duration::from_secs(15)).await;
	}
}
