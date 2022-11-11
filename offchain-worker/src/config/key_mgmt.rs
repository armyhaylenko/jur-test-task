// Use this module to get the keys from local keystore (key id)
// Provide some utility fns for getting public key / account id, and for signing data

use std::sync::Arc;

use primitives::shared::Public;
use sc_keystore::LocalKeystore;
use sp_application_crypto::KeyTypeId;
use sp_keystore::CryptoStore;

#[derive(Debug, PartialEq)]
pub enum KeyError {
	/// Public key is not set for the given KeyTypeId
	PubKeyNotSet,
	/// Unable to get the local keystore
	KeyStoreNotFound,
	/// Provided KeyTypeId is not found in the Keystore
	TypeIdNotFound,
	/// Error related to Keystore
	Other(String),
}

/// Extracts public key from the keystore.
pub async fn get_public_key(
	key_type_id: KeyTypeId,
	local_keystore: &Arc<LocalKeystore>,
) -> Result<Public, KeyError> {
	let local_keys = CryptoStore::sr25519_public_keys(local_keystore.as_ref(), key_type_id).await;
	// if we've inserted a key into the correct keystore, we'll get the first one(just as an
	// arbitrary selection)
	if !local_keys.is_empty() {
		Ok(local_keys[0].into())
	} else {
		log::error!("{:?}", KeyError::PubKeyNotSet);
		Err(KeyError::PubKeyNotSet)
	}
}
