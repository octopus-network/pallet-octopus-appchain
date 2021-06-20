#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

use borsh::{BorshDeserialize, BorshSerialize};
use codec::{Decode, Encode};
use frame_support::traits::OneSessionHandler;
use frame_support::{traits::tokens::fungibles, transactional};
use frame_system::offchain::AppCrypto;
use frame_system::offchain::{
	CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer, SigningTypes,
};
use serde::{de, Deserialize, Deserializer};
use sp_core::{crypto::KeyTypeId, H256};
use sp_io::offchain_index;
use sp_runtime::traits::StaticLookup;
use sp_runtime::{
	offchain::{http, storage::StorageValueRef, Duration},
	traits::{Convert, Hash, IdentifyAccount, Keccak256},
	DigestItem, RuntimeDebug,
};
use sp_std::prelude::*;

pub use pallet::*;

mod motherchain;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"octo");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, KEY_TYPE);
}

/// Identity of an appchain authority.
pub type AuthorityId = crypto::Public;

type AssetBalanceOf<T> =
	<<T as Config>::Assets as fungibles::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

type AssetIdOf<T> =
	<<T as Config>::Assets as fungibles::Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

/// Validator of appchain.
#[derive(Deserialize, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Validator<AccountId> {
	/// The validator's id.
	#[serde(deserialize_with = "deserialize_from_hex_str")]
	#[serde(bound(deserialize = "AccountId: Decode"))]
	id: AccountId,
	/// The weight of this validator in motherchain's staking system.
	// TODO: String -> u128
	weight: u64,
}

fn deserialize_from_hex_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
	S: Decode,
	D: Deserializer<'de>,
{
	let account_id_str: String = Deserialize::deserialize(deserializer)?;
	// TODO
	let account_id_hex =
		hex::decode(&account_id_str[2..]).map_err(|e| de::Error::custom(e.to_string()))?;
	S::decode(&mut &account_id_hex[..]).map_err(|e| de::Error::custom(e.to_string()))
}

/// The validator set of appchain.
#[derive(Deserialize, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ValidatorSet<AccountId> {
	/// The sequence number of this set on the motherchain.
	#[serde(rename = "seq_num")]
	sequence_number: u32,
	/// Validators in this set.
	#[serde(bound(deserialize = "AccountId: Decode"))]
	validators: Vec<Validator<AccountId>>,
}

#[derive(Deserialize, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct LockEvent<AccountId> {
	#[serde(with = "serde_bytes")]
	token_id: Vec<u8>,
	#[serde(deserialize_with = "deserialize_from_hex_str")]
	#[serde(bound(deserialize = "AccountId: Decode"))]
	receiver_id: AccountId,
	#[serde(deserialize_with = "deserialize_from_str")]
	amount: u128,
}

pub fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
	S: sp_std::str::FromStr,
	D: Deserializer<'de>,
	<S as sp_std::str::FromStr>::Err: ToString,
{
	let amount_str: String = Deserialize::deserialize(deserializer)?;
	amount_str.parse::<S>().map_err(|e| de::Error::custom(e.to_string()))
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum Observation<AccountId> {
	UpdateValidatorSet(ValidatorSet<AccountId>),
	LockToken(LockEvent<AccountId>),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct ObservationPayload<Public, BlockNumber, AccountId> {
	public: Public,
	block_number: BlockNumber,
	fact_sequence: u64,
	observation: Observation<AccountId>,
}

impl<T: SigningTypes> SignedPayload<T>
	for ObservationPayload<T::Public, T::BlockNumber, <T as frame_system::Config>::AccountId>
{
	fn public(&self) -> T::Public {
		self.public.clone()
	}
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct XTransferPayload {
	pub token_id: Vec<u8>,
	pub sender: Vec<u8>,
	pub receiver_id: Vec<u8>,
	pub amount: u128,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Message {
	nonce: u64,
	payload: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		CreateSignedTransaction<Call<Self>> + frame_system::Config + pallet_session::Config
	{
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The overarching dispatch call type.
		type Call: From<Call<Self>>;

		type Assets: fungibles::Mutate<<Self as frame_system::Config>::AccountId>;

		// Configuration parameters

		/// A grace period after we send transaction.
		///
		/// To avoid sending too many transactions, we only attempt to send one
		/// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
		/// sending between distinct runs of this offchain worker.
		#[pallet::constant]
		type GracePeriod: Get<Self::BlockNumber>;

		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

		/// The account_id/address of the relay contract on the motherchain.
		const RELAY_CONTRACT: &'static [u8];
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub(super) fn DefaultForAppchainId() -> Vec<u8> {
		Vec::new()
	}

	#[pallet::storage]
	#[pallet::getter(fn appchain_id)]
	pub(super) type AppchainId<T: Config> =
		StorageValue<_, Vec<u8>, ValueQuery, DefaultForAppchainId>;

	/// The current set of validators of this appchain.
	#[pallet::storage]
	pub type CurrentValidatorSet<T: Config> =
		StorageValue<_, ValidatorSet<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	pub type NextValidatorSet<T: Config> = StorageValue<_, ValidatorSet<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	pub type FactSequence<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type Observations<T: Config> =
		StorageMap<_, Twox64Concat, u64, Vec<Observation<T::AccountId>>, ValueQuery>;

	#[pallet::storage]
	pub type Observing<T: Config> = StorageMap<
		_,
		Twox64Concat,
		Observation<T::AccountId>,
		Vec<Validator<T::AccountId>>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type MessageQueue<T: Config> = StorageValue<_, Vec<Message>, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub appchain_id: String,
		pub validators: Vec<(T::AccountId, u64)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { appchain_id: String::new(), validators: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<AppchainId<T>>::put(self.appchain_id.as_bytes());
			Pallet::<T>::initialize_validators(&self.validators);
		}
	}

	#[pallet::event]
	#[pallet::metadata(
		T::AccountId = "AccountId",
		BalanceOfAsset<T> = "Balance",
		AssetIdOfAsset<T> = "AssetId"
	)]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event generated when a new voter votes on a validator set.
		/// \[validator_set, voter\]
		NewVoterFor(ValidatorSet<T::AccountId>, T::AccountId),
		Minted(AssetIdOf<T>, Vec<u8>, T::AccountId, AssetBalanceOf<T>),
		Burned(AssetIdOf<T>, T::AccountId, Vec<u8>, AssetBalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// No CurrentValidatorSet.
		NoCurrentValidatorSet,
		/// The sequence number of new validator set was wrong.
		WrongSequenceNumber,
		/// Must be a validator.
		NotValidator,
		/// Nonce overflow.
		NonceOverflow,
		/// Fact sequence overflow.
		FactSequenceOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Initialization
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			Self::commit()
		}

		/// Offchain Worker entry point.
		///
		/// By implementing `fn offchain_worker` you declare a new offchain worker.
		/// This function will be called when the node is fully synced and a new best block is
		/// succesfuly imported.
		/// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		/// You can use `Local Storage` API to coordinate runs of the worker.
		fn offchain_worker(block_number: T::BlockNumber) {
			let appchain_id = Self::appchain_id();
			if appchain_id.len() == 0 {
				// detach appchain from motherchain when appchain_id == ""
				return;
			}
			let parent_hash = <frame_system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::info!("🐙 Current block: {:?} (parent hash: {:?})", block_number, parent_hash);

			if !Self::should_send(block_number) {
				return;
			}

			let next_seq_num;
			if let Some(cur_val_set) = <CurrentValidatorSet<T>>::get() {
				next_seq_num = cur_val_set.sequence_number + 1;
			} else {
				log::info!("🐙 CurrentValidatorSet must be initialized.");
				return;
			}
			log::info!("🐙 Next validator set sequenc number: {}", next_seq_num);

			let current_fact_sequence = FactSequence::<T>::get();

			// TODO: refactoring into a unified request
			if let Err(e) = Self::fetch_and_update_validator_set(
				block_number,
				appchain_id.clone(),
				current_fact_sequence,
				next_seq_num,
			) {
				log::info!("🐙 fetch_and_update_validator_set: Error: {}", e);
			}
			if let Err(e) = Self::get_and_submit_lock_events(
				block_number,
				appchain_id,
				current_fact_sequence,
				0,
				2,
			) {
				log::info!("🐙 get_and_submit_lock_events: Error: {}", e);
			}
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::submit_observation(ref payload, ref signature) = call {
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into();
				}
				Self::validate_transaction_parameters(
					&payload.block_number,
					payload.fact_sequence,
					payload.public.clone().into_account(),
				)
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submit a new observation.
		///
		/// If the set already exists in the Observations, then the only thing
		/// to do is vote for this set.
		#[pallet::weight(0)]
		pub fn submit_observation(
			origin: OriginFor<T>,
			payload: ObservationPayload<
				T::Public,
				T::BlockNumber,
				<T as frame_system::Config>::AccountId,
			>,
			_signature: T::Signature,
		) -> DispatchResultWithPostInfo {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;
			let cur_val_set =
				<CurrentValidatorSet<T>>::get().ok_or(Error::<T>::NoCurrentValidatorSet)?;
			let who = payload.public.clone().into_account();

			let val = cur_val_set.validators.iter().find(|v| {
				let id = <pallet_session::Module<T>>::key_owner(
					KEY_TYPE,
					&payload.public.clone().into_account().encode(),
				);
				log::info!("🐙 check {:#?} == {:#?}", v.id, id);
				<T as pallet_session::Config>::ValidatorIdOf::convert(v.id.clone()) == id
			});
			if val.is_none() {
				log::info!(
					"🐙 Not a validator in current validator set: {:?}",
					payload.public.clone().into_account()
				);
				return Err(Error::<T>::NotValidator.into());
			}
			let val = val.expect("Validator is valid; qed").clone();

			//
			log::info!(
				"️️️🐙 current_validator_set: {:#?},\nobservation: {:#?},\nwho: {:?}",
				cur_val_set,
				payload.observation,
				who
			);
			//
			<Observations<T>>::mutate(payload.fact_sequence, |obs| {
				let found = obs.iter().any(|o| o == &payload.observation);
				if !found {
					obs.push(payload.observation.clone())
				}
			});
			<Observing<T>>::mutate(&payload.observation, |vals| {
				let found = vals.iter().any(|v| v.id == val.id);
				if !found {
					vals.push(val);
				} else {
					log::info!("🐙 {:?} submits a duplicate ocw tx", val.id);
				}
			});

			let total_weight: u64 = cur_val_set.validators.iter().map(|v| v.weight).sum();
			let weight: u64 =
				<Observing<T>>::get(&payload.observation).iter().map(|v| v.weight).sum();

			// TODO 2/3
			if weight == total_weight {
				match payload.observation {
					Observation::UpdateValidatorSet(val_set) => {
						ensure!(
							val_set.sequence_number == cur_val_set.sequence_number + 1,
							Error::<T>::WrongSequenceNumber
						);

						<NextValidatorSet<T>>::put(val_set);
					}
					Observation::LockToken(event) => {
						// Self::mint_inner(0, vec![], event.receiver_id, 10);
					}
				}

				let obs = <Observations<T>>::get(payload.fact_sequence);
				let _ = obs.iter().map(|o| {
					<Observing<T>>::remove(o);
				});
				<Observations<T>>::remove(payload.fact_sequence);

				FactSequence::<T>::try_mutate(|seq| -> DispatchResultWithPostInfo {
					if let Some(v) = seq.checked_add(1) {
						*seq = v;
					} else {
						return Err(Error::<T>::FactSequenceOverflow.into());
					}

					Ok(().into())
				})?;
			}

			Ok(().into())
		}

		// cross chain transfer

		#[pallet::weight(0)]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			sender_id: Vec<u8>,
			receiver: <T::Lookup as StaticLookup>::Source,
			amount: AssetBalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			Self::mint_inner(asset_id, sender_id, receiver, amount)
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			receiver_id: Vec<u8>,
			amount: AssetBalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			<T::Assets as fungibles::Mutate<T::AccountId>>::burn_from(asset_id, &sender, amount)?;

			let message = XTransferPayload {
				token_id: "USDC.testnet".as_bytes().to_vec(), // TODO
				sender: sender.encode(),
				receiver_id: receiver_id.clone(),
				amount: 41, // TODO
			};

			Self::submit(&sender, &message.try_to_vec().unwrap())?;
			Self::deposit_event(Event::Burned(asset_id, sender, receiver_id, amount));

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn initialize_validators(vals: &Vec<(<T as frame_system::Config>::AccountId, u64)>) {
			if vals.len() != 0 {
				assert!(
					<CurrentValidatorSet<T>>::get().is_none(),
					"CurrentValidatorSet is already initialized!"
				);
				<CurrentValidatorSet<T>>::put(ValidatorSet {
					sequence_number: 0,
					validators: vals
						.iter()
						.map(|x| Validator { id: x.0.clone(), weight: x.1 })
						.collect::<Vec<_>>(),
				});
			}
		}

		fn should_send(block_number: T::BlockNumber) -> bool {
			/// A friendlier name for the error that is going to be returned in case we are in the grace
			/// period.
			const RECENTLY_SENT: () = ();

			// Start off by creating a reference to Local Storage value.
			// Since the local storage is common for all offchain workers, it's a good practice
			// to prepend your entry with the module name.
			let val = StorageValueRef::persistent(b"octopus_appchain::last_send");
			// The Local Storage is persisted and shared between runs of the offchain workers,
			// and offchain workers may run concurrently. We can use the `mutate` function, to
			// write a storage entry in an atomic fashion. Under the hood it uses `compare_and_set`
			// low-level method of local storage API, which means that only one worker
			// will be able to "acquire a lock" and send a transaction if multiple workers
			// happen to be executed concurrently.
			let res = val.mutate(|last_send: Option<Option<T::BlockNumber>>| {
				// We match on the value decoded from the storage. The first `Option`
				// indicates if the value was present in the storage at all,
				// the second (inner) `Option` indicates if the value was succesfuly
				// decoded to expected type (`T::BlockNumber` in our case).
				match last_send {
					// If we already have a value in storage and the block number is recent enough
					// we avoid sending another transaction at this time.
					Some(Some(block)) if block_number < block + T::GracePeriod::get() => {
						Err(RECENTLY_SENT)
					}
					// In every other case we attempt to acquire the lock and send a transaction.
					_ => Ok(block_number),
				}
			});

			// The result of `mutate` call will give us a nested `Result` type.
			// The first one matches the return of the closure passed to `mutate`, i.e.
			// if we return `Err` from the closure, we get an `Err` here.
			// In case we return `Ok`, here we will have another (inner) `Result` that indicates
			// if the value has been set to the storage correctly - i.e. if it wasn't
			// written to in the meantime.
			match res {
				// The value has been set correctly, which means we can safely send a transaction now.
				Ok(Ok(_block_number)) => true,
				// We are in the grace period, we should not send a transaction this time.
				Err(RECENTLY_SENT) => false,
				// We wanted to send a transaction, but failed to write the block number (acquire a
				// lock). This indicates that another offchain worker that was running concurrently
				// most likely executed the same logic and succeeded at writing to storage.
				// Thus we don't really want to send the transaction, knowing that the other run
				// already did.
				Ok(Err(_)) => false,
			}
		}

		fn fetch_and_update_validator_set(
			block_number: T::BlockNumber,
			appchain_id: Vec<u8>,
			current_fact_sequence: u64,
			next_seq_num: u32,
		) -> Result<(), &'static str> {
			log::info!("🐙 in fetch_and_update_validator_set");

			// Make an external HTTP request to fetch the current validator set.
			// Note this call will block until response is received.
			let next_val_set =
				Self::fetch_validator_set(T::RELAY_CONTRACT.to_vec(), appchain_id, next_seq_num)
					.map_err(|_| "Failed to fetch validator set")?;
			log::info!("🐙 new validator set: {:#?}", next_val_set);

			// -- Sign using any account
			let (_, result) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| ObservationPayload {
						public: account.public.clone(),
						block_number,
						fact_sequence: current_fact_sequence,
						observation: Observation::UpdateValidatorSet(next_val_set.clone()),
					},
					|payload, signature| Call::submit_observation(payload, signature),
				)
				.ok_or("🐙 No local accounts accounts available.")?;
			result.map_err(|()| "🐙 Unable to submit transaction")?;

			Ok(())
		}

		fn get_and_submit_lock_events(
			block_number: T::BlockNumber,
			appchain_id: Vec<u8>,
			current_fact_sequence: u64,
			start: u32,
			limit: u32,
		) -> Result<(), &'static str> {
			log::info!("🐙 in get_and_submit_lock_events");

			let events =
				Self::get_locked_events(T::RELAY_CONTRACT.to_vec(), appchain_id, start, limit)
					.map_err(|_| "Failed to get lock events")?;
			log::info!("🐙 new lock events: {:#?}", events);

			// -- Sign using any account
			let (_, result) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					|account| ObservationPayload {
						public: account.public.clone(),
						block_number,
						fact_sequence: current_fact_sequence,
						// TODO
						observation: Observation::LockToken(events[0].clone()),
					},
					|payload, signature| Call::submit_observation(payload, signature),
				)
				.ok_or("🐙 No local accounts accounts available.")?;
			result.map_err(|()| "🐙 Unable to submit transaction")?;

			Ok(())
		}

		fn mint_inner(
			asset_id: AssetIdOf<T>,
			sender_id: Vec<u8>,
			receiver: <T::Lookup as StaticLookup>::Source,
			amount: AssetBalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let receiver = T::Lookup::lookup(receiver)?;
			<T::Assets as fungibles::Mutate<T::AccountId>>::mint_into(asset_id, &receiver, amount)?;
			Self::deposit_event(Event::Minted(asset_id, sender_id, receiver, amount));

			Ok(().into())
		}

		fn validate_transaction_parameters(
			block_number: &T::BlockNumber,
			fact_sequence: u64,
			account_id: <T as frame_system::Config>::AccountId,
		) -> TransactionValidity {
			// Let's make sure to reject transactions from the future.
			let current_block = <frame_system::Pallet<T>>::block_number();
			if &current_block < block_number {
				log::info!(
					"🐙 InvalidTransaction => current_block: {:?}, block_number: {:?}",
					current_block,
					block_number
				);
				return InvalidTransaction::Future.into();
			}

			ValidTransaction::with_tag_prefix("OctopusAppchain")
				// We set base priority to 2**20 and hope it's included before any other
				// transactions in the pool. Next we tweak the priority depending on the
				// sequence of the fact that happened on motherchain.
				.priority(T::UnsignedPriority::get().saturating_add(fact_sequence))
				// This transaction does not require anything else to go before into the pool.
				//.and_requires()
				// One can only vote on the validator set with the same seq_num once.
				.and_provides((fact_sequence, account_id))
				// The transaction is only valid for next 5 blocks. After that it's
				// going to be revalidated by the pool.
				.longevity(5)
				// It's fine to propagate that transaction to other peers, which means it can be
				// created even by nodes that don't produce blocks.
				// Note that sometimes it's better to keep it for yourself (if you are the block
				// producer), since for instance in some schemes others may copy your solution and
				// claim a reward.
				.propagate(true)
				.build()
		}

		fn submit(_who: &T::AccountId, payload: &[u8]) -> DispatchResultWithPostInfo {
			Nonce::<T>::try_mutate(|nonce| -> DispatchResultWithPostInfo {
				if let Some(v) = nonce.checked_add(1) {
					*nonce = v;
				} else {
					return Err(Error::<T>::NonceOverflow.into());
				}

				MessageQueue::<T>::append(Message { nonce: *nonce, payload: payload.to_vec() });
				Ok(().into())
			})
		}

		fn commit() -> Weight {
			let messages: Vec<Message> = MessageQueue::<T>::take();
			if messages.is_empty() {
				return 0;
			}

			let commitment_hash = Self::make_commitment_hash(&messages);

			<frame_system::Pallet<T>>::deposit_log(DigestItem::Other(
				commitment_hash.as_bytes().to_vec(),
			));

			let key = Self::make_offchain_key(commitment_hash);
			offchain_index::set(&*key, &messages.encode());

			0
		}

		fn make_commitment_hash(messages: &[Message]) -> H256 {
			let messages: Vec<_> =
				messages.iter().map(|message| (message.nonce, message.payload.clone())).collect();
			let input = messages.encode();
			Keccak256::hash(&input)
		}

		fn make_offchain_key(hash: H256) -> Vec<u8> {
			(b"commitment", hash).encode()
		}
	}

	pub type SessionIndex = u32;

	impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
		fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log::info!(
				"🐙 [{:?}] planning new_session({})",
				<frame_system::Pallet<T>>::block_number(),
				new_index
			);

			let next_val_set = <NextValidatorSet<T>>::get();
			match next_val_set {
				Some(new_val_set) => {
					<CurrentValidatorSet<T>>::put(new_val_set.clone());
					<CurrentValidatorSet<T>>::kill();
					log::info!("🐙 validator set changed to: {:#?}", new_val_set.clone());
					Some(new_val_set.validators.into_iter().map(|vals| vals.id).collect())
				}
				None => {
					log::info!("🐙 validator set has't changed");
					None
				}
			}
		}

		fn start_session(start_index: SessionIndex) {
			log::info!(
				"🐙 [{:?}] starting start_session({})",
				<frame_system::Pallet<T>>::block_number(),
				start_index
			);
		}

		fn end_session(end_index: SessionIndex) {
			log::info!(
				"🐙 [{:?}] ending end_session({})",
				<frame_system::Pallet<T>>::block_number(),
				end_index
			);
		}
	}

	impl<T: Config> sp_runtime::BoundToRuntimeAppPublic for Pallet<T> {
		type Public = AuthorityId;
	}

	impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
		type Key = AuthorityId;

		fn on_genesis_session<'a, I: 'a>(_authorities: I)
		where
			I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
		{
			// ignore
		}

		fn on_new_session<'a, I: 'a>(_changed: bool, _validators: I, _queued_validators: I)
		where
			I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
		{
			// ignore
		}

		fn on_disabled(_i: usize) {
			// ignore
		}
	}
}
