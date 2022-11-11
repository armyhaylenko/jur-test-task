#![cfg_attr(not(feature = "std"), no_std)]

use core::time::Duration;
use sp_core::{crypto::AccountId32, H160};

// We will run offchain resolution for JUR balances on Thor every X seconds
pub const PROCESSING_INTERVAL: Duration = Duration::from_secs(6);

// We will store the workload in the offchain db to
pub const WORKLOAD_KEY: &[u8] = b"workload";

// Types shared across runtime and client
pub mod shared {
	use super::*;
	use codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use sp_application_crypto::{sr25519, KeyTypeId};

	pub type Hash = sp_core::H256;
	pub type BlockNumber = u32;

	pub const PUBLIC_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"jurK");
	sp_application_crypto::app_crypto!(sr25519, PUBLIC_KEY_TYPE_ID);

	#[derive(Encode, Decode, Clone, Debug, PartialEq, TypeInfo)]
	pub enum Call<BlockNumber, AccountId> {
		SubmitBalancesData {
			block_num: BlockNumber,
			chain_account: AccountId,
			thor_account: H160,
			balance: u128,
		},
	}
}

pub mod runtime_api {
	use super::{
		shared::{Public, Signature},
		*,
	};
	use codec::{Decode, Encode};
	use sp_core::sp_std::vec::Vec;

	#[derive(Encode, Decode, PartialEq, Debug)]
	pub enum Error {
		AccountConversion,
	}

	sp_api::decl_runtime_apis! {
		pub trait BalancesOffchainWorkerApi {
			fn submit_balance(
				call: Vec<u8>,
				signature: Signature,
				public: Public,
			) -> Result<(), ()>;
		}

		pub trait WorkloadQueryApi {
			fn get_workload() -> Vec<(AccountId32, H160)>;
		}
	}
}
