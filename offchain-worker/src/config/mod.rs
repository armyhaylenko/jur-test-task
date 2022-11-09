pub mod config_provider;
pub mod key_mgmt;

use crate::PluginError;
use primitives::shared::Pair;
use sc_keystore::LocalKeystore;
use std::sync::Arc;

/// Get keypair.
///
/// This function extracts the node's keys from `LocalKeystore`
pub async fn get_keypair(keystore: &Arc<LocalKeystore>) -> Result<Arc<Pair>, PluginError> {
	let public = config_provider::get_key_info(&keystore.clone()).await;
	let key = Arc::new(
		keystore
			.key_pair::<Pair>(&public)
			.map_err(|_| PluginError::KeystoreError)?
			.ok_or(PluginError::KeyNotFound)?,
	);
	Ok(key)
}
