#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use codec::Decode;
	use frame_support::pallet_prelude::*;
	use frame_system::{offchain::SendTransactionTypes, pallet_prelude::*};
	use sp_core::H160;
	use sp_runtime::traits::{AppVerify, IdentifyAccount, Verify};
	use sp_std::prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// A static message that has to be signed in Ethereum context
		/// and later verified as part of the signature.
		#[pallet::constant]
		type MessageToSign: Get<[u8; 16]>;

		/// A Signature can be verified with a specific `PublicKey`.
		/// The additional traits are boilerplate.
		type Signature: Verify<Signer = Self::VCTPublicKey> + Encode + Decode + Parameter;

		/// A PublicKey can be converted into an `AccountId`. This is required by the
		/// `Signature` type.
		/// The additional traits are boilerplate.
		/// VCT here means VeChainThor.
		type VCTPublicKey: IdentifyAccount<AccountId = Self::VCTPublicKey>
			+ Encode
			+ Decode
			+ Parameter;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	/// A storage that holds future workload for offchain workers.
	/// This will be cleaned up after successfully syncing balances.
	#[pallet::storage]
	#[pallet::getter(fn get_workload)]
	pub type Workload<T> = StorageMap<_, Identity, <T as frame_system::Config>::AccountId, H160>;

	/// A mock balances storage. Can be later replaced by `Currency` trait, though
	/// as a PoC this storage is fine.
	#[pallet::storage]
	#[pallet::getter(fn get_balances)]
	pub type UserBalances<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, u128, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Request balance sync.
		///
		/// # Parameters
		/// * `who` - who requested the sync
		/// * `vct_public` - public key of target account on VeChainThor
		ClaimCreated { who: T::AccountId, vct_public: T::VCTPublicKey },

		/// Synced locked balances on VeChainThor to this chain.
		///
		/// # Parameters
		/// * `chain_account` - the target account on this chain
		/// * `thor_account` - the source account on VeChainThor
		/// * `balance` - amount of synced balances
		BalanceDataStored {
			chain_account: <T as frame_system::Config>::AccountId,
			thor_account: H160,
			balance: u128,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Errored when verifying claim signature
		InvalidSignature,
		/// Could not convert **ecdsa** public key to Ethereum address
		PubkeyToAddressConversionFailure,
		/// Already started syncing balances
		AlreadyStarted,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create claim to sync VeChainThor balances to this chain.
		/// This will check for locked balances in the target contract
		/// and sync them to a storage item in this pallet.
		///
		/// Example compressed public that has JUR tokens on balance:
		/// 0x03fe89383702c1223358b6b613d35b439ec2b926407e9a5e9fce80273843393363
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())] // TODO: add weight
		pub fn create_claim(
			origin: OriginFor<T>,
			vct_public: T::VCTPublicKey,
			_signature: T::Signature,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Workload::<T>::get(who.clone()).is_none(), Error::<T>::AlreadyStarted);
			// TODO: uncomment. Since we don't have a frontend to conveniently produce signature,
			// TODO: we don't check it at all rn.

			// let who_nonce = frame_system::Account::<T>::get(who.clone()).nonce;
			// let message_to_sign: Vec<u8> =
			// 	[who.clone().encode(), who_nonce.encode(), T::MessageToSign::get().to_vec()]
			// 		.encode();
			// ensure!(signature.verify(&*message_to_sign, &vct_public),
			// Error::<T>::InvalidSignature);

			let uncompressed =
				libsecp256k1::PublicKey::parse_compressed(&vct_public.encode().try_into().unwrap())
					.map_err(|_| Error::<T>::PubkeyToAddressConversionFailure)?
					.serialize();

			let vct_address: [u8; 20] = sp_io::hashing::keccak_256(&uncompressed[1..])[12..]
				.try_into()
				.map_err(|_| Error::<T>::PubkeyToAddressConversionFailure)?;
			let vct_h160: H160 = vct_address.into();

			Workload::<T>::insert(who.clone(), vct_h160);
			Self::deposit_event(Event::ClaimCreated { who, vct_public });
			Ok(())
		}

		/// Submit balances query result.
		/// This dispatchable has to be called by an offchain worker
		/// that has processed some workload item.
		///
		/// The reason for the following parameters is that we need
		/// to verify that the offchain worker has sent the transaction, not someone else,
		/// since the transaction is unsigned.
		#[pallet::weight(10_000)] // TODO: change weight
		pub fn submit_user_balance(
			origin: OriginFor<T>,
			payload: Vec<u8>,
			_signature: primitives::shared::Signature,
			_public: primitives::shared::Public,
		) -> DispatchResult {
			ensure_none(origin)?;
			let call =
				primitives::shared::Call::<T::BlockNumber, T::AccountId>::decode(&mut &payload[..])
					.unwrap();
			let primitives::shared::Call::SubmitBalancesData {
				block_num: block_number,
				chain_account,
				thor_account,
				balance,
			} = call;

			Workload::<T>::remove(chain_account.clone());

			log::info!(
				"Received values from offchain worker: {:?}",
				(block_number, chain_account.clone(), thor_account, balance)
			);

			UserBalances::<T>::insert(chain_account.clone(), balance);

			Self::deposit_event(Event::<T>::BalanceDataStored {
				chain_account,
				thor_account: thor_account.clone(),
				balance,
			});

			Ok(())
		}
	}

	impl<T> Pallet<T>
	where
		T: Config + SendTransactionTypes<Call<T>>,
	{
		#[allow(clippy::result_unit_err)]
		pub fn create_extrinsic_from_external_call(
			payload: Vec<u8>,
			public: primitives::shared::Public,
			signature: primitives::shared::Signature,
		) -> Result<(), ()>
		where
			T::Hash: From<sp_core::H256>,
		{
			use frame_system::offchain::SubmitTransaction;
			let external_call =
				primitives::shared::Call::<T::BlockNumber, T::AccountId>::decode(&mut &payload[..])
					.map_err(|_| ())?;
			let call = match external_call {
				primitives::shared::Call::SubmitBalancesData { .. } => Call::submit_user_balance {
					payload,
					signature: signature.into(),
					public: public.into(),
				},
			};
			let result =
				SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.clone().into());

			match result {
				Ok(()) => log::info!(
					target: "runtime::template",
					"Submitted user balances {:?}",
					call
				),
				Err(e) => log::error!(
					target: "runtime::template",
					"Error submitting balances ({:?}): {:?}",
					call,
					e,
				),
			}
			result
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::submit_user_balance { public, payload, signature } => {
					let decoded_call: primitives::shared::Call<T::BlockNumber, T::AccountId> =
						primitives::shared::Call::decode(&mut &payload[..])
							.map_err(|_| InvalidTransaction::Call)?;
					let primitives::shared::Call::SubmitBalancesData { chain_account, .. } =
						decoded_call;
					if signature.verify(&payload[..], &public) {
						ValidTransaction::with_tag_prefix("Template")
							.priority(TransactionPriority::MAX)
							// The transaction is only valid for next 5 blocks. After that it's
							// going to be revalidated by the pool.
							.longevity(5 as u64)
							.propagate(true)
							// dedup by chain account to reject potential modifications by other
							// offchain workers
							.and_provides(chain_account)
							.build()
					} else {
						InvalidTransaction::Call.into()
					}
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}
