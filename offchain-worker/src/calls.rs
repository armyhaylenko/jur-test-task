use codec::Encode;
use primitives::{
	runtime_api::{BalancesOffchainWorkerApi, WorkloadQueryApi},
	shared::{Call, Pair},
};
use sc_client_api::HeaderBackend;
use sp_api::{BlockT, HeaderT, ProvideRuntimeApi};
use sp_core::{crypto::AccountId32, Pair as _, H160};
use sp_runtime::generic;
use std::sync::Arc;

/// Submit a call to the runtime.
///
/// This function is used to send a variant of `MapToCall`
/// to the runtime. Upon submission, it will be processed by the
/// respective runtime api impl in `runtime` and dispatched to
/// the respective pallet.
pub fn submit_call<B, C: 'static>(
	client: Arc<C>,
	pair: Arc<Pair>,
	mapped_call: Call<<<B as BlockT>::Header as HeaderT>::Number, AccountId32>,
) -> Result<(), ()>
where
	B: BlockT,
	C: ProvideRuntimeApi<B> + HeaderBackend<B>,
	C::Api: BalancesOffchainWorkerApi<B>,
{
	let best_hash = client.info().best_hash;
	let payload = mapped_call.encode();
	let signature = pair.sign(&payload);
	client
		.runtime_api()
		// Submit our call to the runtime api
		.submit_balance(&generic::BlockId::Hash(best_hash), payload, signature, pair.public())
		.map_err(|_| ())?
}

pub fn get_workload<B, C: 'static>(
	client: Arc<C>,
	best_number: <<B as BlockT>::Header as HeaderT>::Number,
) -> Result<Vec<(AccountId32, H160)>, ()>
where
	B: BlockT,
	C: ProvideRuntimeApi<B> + HeaderBackend<B>,
	C::Api: WorkloadQueryApi<B>,
{
	client
		.runtime_api()
		// Submit our call to the runtime api
		.get_workload(&generic::BlockId::Number(best_number))
		.map_err(|_| ())
}
