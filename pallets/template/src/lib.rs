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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::{string::String, *};
	use sp_core::{
		offchain::{Duration, HttpError},
		H160,
	};
	use sp_runtime::{
		offchain::http,
		traits::{IdentifyAccount, Verify},
	};
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type MessageToSign: Get<[u8; 16]>;

		/// A Signature can be verified with a specific `PublicKey`.
		/// The additional traits are boilerplate.
		type Signature: Verify<Signer = Self::VCTPublicKey> + Encode + Decode + Parameter;

		/// A PublicKey can be converted into an `AccountId`. This is required by the
		/// `Signature` type.
		/// The additional traits are boilerplate.
		type VCTPublicKey: IdentifyAccount<AccountId = Self::VCTPublicKey>
			+ Encode
			+ Decode
			+ Parameter;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Workload<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, H160>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		InvalidSignature,
		PubkeyToAddressConversionFailure,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn offchain_worker(_n: T::BlockNumber) {
			for (this_chain_account, thor_account) in Workload::<T>::iter() {
				let balance_bytes = Self::fetch_jur_balance(&thor_account).unwrap_or_else(|_| {
					log::warn!("Could not get balance of {:?} on VeChain", &thor_account,);
					vec![0]
				});
				let balance: &str = sp_std::str::from_utf8(&balance_bytes).unwrap_or_else(|_| {
					log::warn!("No UTF8 body");
					"0.5234234"
				});
				log::info!("Account {:?} has initiated the transfer of their JUR tokens from VeChain account {}, balance: {}",
					this_chain_account, hex::encode(thor_account.0), balance);
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())] // TODO: add weight
		pub fn create_claim(
			origin: OriginFor<T>,
			vct_public: T::VCTPublicKey,
			signature: T::Signature,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;
			// // TODO: uncomment
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

			Workload::<T>::insert(who, vct_h160);
			// // Emit an event.
			// Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// // Read a value from storage.
			// match <Something<T>>::get() {
			// 	// Return an error if the value has not been set.
			// 	None => return Err(Error::<T>::NoneValue.into()),
			// 	Some(old) => {
			// 		// Increment the value read from storage; will error in the event of overflow.
			// 		let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			// 		// Update the value in storage with the incremented result.
			// 		<Something<T>>::put(new);
			// 		Ok(())
			// 	},
			// }
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn fetch_jur_balance(addr: &H160) -> Result<Vec<u8>, http::Error> {
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
			let hex_encoded_addr = hex::encode(addr.0);
			let request_address: String =
				format!("http://localhost:3000/balance?account=0x{}", hex_encoded_addr.clone());
			let request = http::Request::get(&request_address);
			let pending = request.deadline(deadline).send().map_err(|e| match e {
				HttpError::DeadlineReached => http::Error::DeadlineReached,
				HttpError::IoError => http::Error::IoError,
				HttpError::Invalid => http::Error::Unknown,
			})?;
			let response = pending
				.try_wait(deadline)
				.or_else(|pending| pending.try_wait(deadline))
				.map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
			}
			// Next we want to fully read the response body and collect it to a vector of bytes.
			// Note that the return object allows you to read the body in chunks as well
			// with a way to control the deadline.
			let balance_bytes = response.body().collect::<Vec<u8>>();

			Ok(balance_bytes)
		}
	}
}
