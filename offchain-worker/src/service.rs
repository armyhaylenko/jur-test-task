use futures::StreamExt;
use primitives::{
	runtime_api::{BalancesOffchainWorkerApi, WorkloadQueryApi},
	PROCESSING_INTERVAL,
};
use sc_client_api::HeaderBackend;
use sc_client_db::offchain::LocalStorage;
use sc_keystore::LocalKeystore;
use sp_api::{BlockT, ProvideRuntimeApi};
use std::sync::Arc;
use tokio::{
	sync::Mutex,
	time::{interval, Interval},
};

use crate::{
	calls::{get_workload, submit_call},
	config::get_keypair,
	offchain::{has_key, store_key},
	request,
};
use primitives::shared::{Call, Pair};

// Start the module. To be initiated by the node's service.
// In here we use a runtime interface, which consists of some logic running on an interval
// as well as some business logic that retrieves the offchain data.
pub async fn start<B, C: 'static>(
	// Accept some closure that expects a `MapToCall`
	client: Arc<C>,
	offchain_storage: Arc<Mutex<LocalStorage>>,
	keystore: Arc<LocalKeystore>,
) where
	B: BlockT,
	C: ProvideRuntimeApi<B> + HeaderBackend<B>,
	C::Api: BalancesOffchainWorkerApi<B> + WorkloadQueryApi<B>,
{
	// Indicate some arbitrary seconds interval, where for each "tick" the business logic will be
	// invoked
	// Fetch a JSON object with various values configured by the node operator. In addition, it
	// contains the first valid public key set by the node operator, set in the key
	// `config_account_id`.
	let interval = interval(PROCESSING_INTERVAL);
	let pair = get_keypair(&keystore).await.expect("Could not get pair from the keystore");
	run_service::<B, C>(client, pair, interval, &offchain_storage).await
}

/// In here, we run our initial logic related to the logic provider flow.
/// This includes generating the envelope id, creating operating envelope
/// hash, committing the hash and saving the data in the offchain storage
/// for further reveals.
async fn run_service<B, C: 'static>(
	client: Arc<C>,
	pair: Arc<Pair>,
	interval: Interval,
	offchain_storage: &Arc<Mutex<LocalStorage>>,
) where
	B: BlockT,
	C: ProvideRuntimeApi<B> + HeaderBackend<B>,
	C::Api: BalancesOffchainWorkerApi<B> + WorkloadQueryApi<B>,
{
	tokio_stream::wrappers::IntervalStream::new(interval)
		.for_each(|_| {
			let client = client.clone();
			let pair = pair.clone();
			let offchain_storage = offchain_storage.clone();
			let best_number = client.info().best_number;
			let workload = get_workload(client.clone(), best_number);
			async move {
				let workload = if let Ok(workload) = workload {
					workload
				} else {
					log::error!(target: "offchain-worker-service", "Workload runtime API returned Error!");
					return
				};
				for (chain_addr, thor_addr) in workload {
					if has_key(chain_addr.clone(), &offchain_storage).await {
						continue;
					};
					let fetch_result = request::fetch_jur_balance(&thor_addr).await;
					if let Ok(balance) = fetch_result {
						let call = Call::SubmitBalancesData {
							block_num: best_number,
							chain_account: chain_addr.clone(),
							thor_account: thor_addr,
							balance,
						};

						if submit_call(client.clone(), pair.clone(), call).is_ok() {
							// Store the relevant workload account
							store_key(chain_addr, &offchain_storage).await;
						}
					} else {
						log::error!(target: "offchain-worker-service", "Could not fetch user balances, error: {:?}", fetch_result.unwrap_err());
					}
				}
			}
		})
		.await
}
