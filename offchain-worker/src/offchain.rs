use codec::{Decode, Encode};
use primitives::WORKLOAD_KEY;
use sc_client_db::offchain::LocalStorage;
use sp_core::{
	crypto::AccountId32,
	offchain::{OffchainStorage, STORAGE_PREFIX},
};
use std::sync::Arc;
use tokio::sync::Mutex;

// Set a key for the given Thor account to avoid reprocessing
pub async fn store_key(key: AccountId32, offchain_storage: &Arc<Mutex<LocalStorage>>) {
	let mut lock = offchain_storage.lock().await;
	// If we've already set some keys for the logic to track
	if let Some(workload) = lock.get(STORAGE_PREFIX, WORKLOAD_KEY) {
		match Vec::<AccountId32>::decode(&mut &workload[..]) {
			Ok(mut keys_list) =>
				if !keys_list.contains(&key) {
					keys_list.push(key);
					lock.set(STORAGE_PREFIX, WORKLOAD_KEY, &keys_list.encode());
				},
			Err(err) => log::error!("Error when decoding storage value: {:?}", err),
		}
	} else {
		lock.set(STORAGE_PREFIX, WORKLOAD_KEY, &vec![key].encode());
	}
}

// Set a key for the given Thor account to avoid reprocessing
pub async fn has_key(key: AccountId32, offchain_storage: &Arc<Mutex<LocalStorage>>) -> bool {
	let lock = offchain_storage.lock().await;
	// If we've already set some keys for the logic to track
	if let Some(workload) = lock.get(STORAGE_PREFIX, WORKLOAD_KEY) {
		match Vec::<AccountId32>::decode(&mut &workload[..]) {
			Ok(keys_list) =>
				keys_list.contains(&key),
			Err(_) => false,
		}
	} else {
		false
	}
}
