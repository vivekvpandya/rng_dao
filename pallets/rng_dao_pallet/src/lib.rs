#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

//NOTE: Generate hash of secret number (u64) based on its little_endian representation as array of
//u8 (byte)

#[frame_support::pallet]
pub mod pallet {
	use crate::weights::RngDaoWeightInfo;
	use codec::FullCodec;
	use core::fmt::Debug;
	use frame_support::{
		ensure, pallet_prelude::*, traits::fungible::Transfer, PalletId, RuntimeDebug,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::{
		traits::{
			AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedDiv, CheckedSub, Hash,
			Keccak256, One, Zero,
		},
		ArithmeticError, SaturatedConversion,
	};

	#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo)]
	pub struct RngCycle<AccountId, Balance, BlockNumber, RandomNumber> {
		pub creator: AccountId,
		pub bounty: Balance,
		pub started: BlockNumber,
		/// random_number is only valid if Status is `CompletedWithSuccess`
		pub random_number: RandomNumber,
		pub generators_count: u8,
		pub revealed_count: u8,
	}

	#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo)]
	pub struct Generator {
		secret: u64,
		hash: H256,
		is_bot: bool,
	}

	pub(crate) type BalanceOf<T> = <T as Config>::Balance;
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type RngCycleOf<T> = RngCycle<AccountIdOf<T>, BalanceOf<T>, BlockNumberOf<T>, u64>;
	pub(crate) type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type CycleId: FullCodec
			+ MaxEncodedLen
			+ Default
			+ Debug
			+ TypeInfo
			+ CheckedAdd
			+ One
			+ Sized
			+ Zero
			+ Clone
			+ Copy
			+ Eq
			+ PartialEq;
		type Balance: FullCodec
			+ MaxEncodedLen
			+ Default
			+ Debug
			+ TypeInfo
			+ AtLeast32BitUnsigned
			+ CheckedAdd
			+ CheckedSub
			+ CheckedDiv;

		type Balances: Transfer<AccountIdOf<Self>, Balance = BalanceOf<Self>>;

		#[pallet::constant]
		type MinBounty: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type DelayBeforeBots: Get<BlockNumberOf<Self>>;

		#[pallet::constant]
		type DelayBeforeSecondPhase: Get<BlockNumberOf<Self>>;

		#[pallet::constant]
		type SecondPhaseDuration: Get<BlockNumberOf<Self>>;

		#[pallet::constant]
		type MaxGenerators: Get<u8>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type WeightInfo: RngDaoWeightInfo;
	}

	#[pallet::type_value]
	pub fn CycleCountOnEmpty<T: Config>() -> T::CycleId {
		Zero::zero()
	}

	#[pallet::storage]
	#[pallet::getter(fn get_cycle_count)]
	pub type CycleCount<T: Config> = StorageValue<_, T::CycleId, ValueQuery, CycleCountOnEmpty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn cycles)]
	pub type Cycles<T: Config> = StorageMap<_, Blake2_128Concat, T::CycleId, RngCycleOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn generators)]
	pub type Generators<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::CycleId,
		Blake2_128Concat,
		AccountIdOf<T>,
		Generator,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CycleCreated { bounty: T::Balance, creator: T::AccountId },
		HashReceived { cycle_id: T::CycleId, sender: T::AccountId, hash: H256 },
		SecretReceived { cycle_id: T::CycleId, sender: T::AccountId },
		CycleCompleted { cycle_id: T::CycleId, creator: T::AccountId, random_number: u64 },
		CycleFailed { cycle_id: T::CycleId, creator: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		BountyMustBeGreaterThanMinBounty,
		NoCycleFound,
		BotsNotAllowedYet,
		MaxGeneratorsReached,
		SecondPhaseNotStartedYet,
		NotAuthorizedToGetRandomNumber,
		RandomNumberNotYetGenerated,
		SecretDoesNotMatchHash,
		NotSubmitedHashInFirstPhase,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_new_rng_cycle())]
		pub fn create_new_rng_cycle(origin: OriginFor<T>, bounty: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(bounty >= T::MinBounty::get(), Error::<T>::BountyMustBeGreaterThanMinBounty);
			let cycle_id =
				CycleCount::<T>::try_mutate(|cycle_count| -> Result<T::CycleId, DispatchError> {
					let cycle_id = *cycle_count;

					Cycles::<T>::insert(
						cycle_id.clone(),
						RngCycleOf::<T> {
							creator: who.clone(),
							bounty: bounty.clone(),
							started: <frame_system::Pallet<T>>::block_number(),
							random_number: 0_u64,
							generators_count: 0_u8,
							revealed_count: 0_u8,
						},
					);
					*cycle_count = cycle_id
						.checked_add(&T::CycleId::one())
						.ok_or(ArithmeticError::Overflow)?;
					Ok(cycle_id)
				})?;
			T::Balances::transfer(&who, &Self::account_id(&cycle_id), bounty.clone(), true)?;

			Self::deposit_event(Event::CycleCreated { bounty, creator: who });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::send_hash())]
		pub fn send_hash(
			origin: OriginFor<T>,
			cycle_id: T::CycleId,
			hash: sp_core::H256,
			is_bot: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Cycles::<T>::try_mutate(cycle_id, |cycle| -> DispatchResult {
				let mut cycle = cycle.as_mut().ok_or(Error::<T>::NoCycleFound)?;
				// check for MaxGeneratorsReached error
				let max_genrators: u8 = T::MaxGenerators::get();
				ensure!(
					cycle.generators_count + 1 <= max_genrators,
					Error::<T>::MaxGeneratorsReached
				);
				cycle.generators_count += 1;
				let now = <frame_system::Pallet<T>>::block_number();
				// bots can participate only after some delay
				ensure!(
					!is_bot || now > cycle.started + T::DelayBeforeBots::get(),
					Error::<T>::BotsNotAllowedYet
				);
				let generator = Generator { secret: 0_u64, hash, is_bot };
				Generators::<T>::insert(cycle_id, who.clone(), generator);

				T::Balances::transfer(
					&who,
					&Self::account_id(&cycle_id.clone()),
					T::Deposit::get(),
					true,
				)?;
				Ok(())
			})?;

			Self::deposit_event(Event::HashReceived { cycle_id, sender: who, hash });
			Ok(())
		}

		/// As soon as generator reveals correct secret, his/her deposit + reward is returned.
		/// If secret is different than hash commited in first phase then he/she looses deposit.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::reveal_secret())]
		pub fn reveal_secret(
			origin: OriginFor<T>,
			cycle_id: T::CycleId,
			secret: u64,
			_is_bot: bool,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();
			Cycles::<T>::try_mutate(cycle_id, |cycle| -> DispatchResult {
				let mut cycle = cycle.as_mut().ok_or(Error::<T>::NoCycleFound)?;
				let second_phase_start =
					cycle.started + T::DelayBeforeBots::get() + T::DelayBeforeSecondPhase::get();
				ensure!(now >= second_phase_start, Error::<T>::SecondPhaseNotStartedYet);
				let generator = Generators::<T>::get(cycle_id, who.clone())
					.ok_or(Error::<T>::NotSubmitedHashInFirstPhase)?;
				// compute hash and see if they matches
				let bytes = secret.to_le_bytes();
				let hash = Keccak256::hash(&bytes);
				if hash == generator.hash {
					// reward the generator and increment revealed_count
					// update random_number
					cycle.revealed_count += 1;
					cycle.random_number ^= secret;
					let total_shares = cycle.generators_count + 1; // add one for our profit share
					let share = cycle
						.bounty
						.checked_div(&total_shares.saturated_into())
						.ok_or(ArithmeticError::Underflow)?;
					let transfer_value = share
						.checked_add(&T::Deposit::get().saturated_into())
						.ok_or(ArithmeticError::Overflow)?;

					T::Balances::transfer(
						&Self::account_id(&cycle_id.clone()),
						&who,
						transfer_value,
						true,
					)?;
					Self::deposit_event(Event::SecretReceived { cycle_id, sender: who.clone() });
					Ok(())
				} else {
					return Err(Error::<T>::SecretDoesNotMatchHash.into())
				}
			})?;
			// remove generator from storage
			Generators::<T>::remove(cycle_id, who);
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::get_random_number())]
		pub fn get_random_number(origin: OriginFor<T>, cycle_id: T::CycleId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// only creator can execute this extrinsic
			let cycle = Cycles::<T>::get(cycle_id).ok_or(Error::<T>::NoCycleFound)?;
			ensure!(cycle.creator == who, Error::<T>::NotAuthorizedToGetRandomNumber);
			let now = <frame_system::Pallet<T>>::block_number();
			let second_phase_start =
				cycle.started + T::DelayBeforeBots::get() + T::DelayBeforeSecondPhase::get();
			let finish = second_phase_start + T::SecondPhaseDuration::get();
			ensure!(now >= finish, Error::<T>::RandomNumberNotYetGenerated);
			if cycle.generators_count == 0 || cycle.revealed_count == 0 {
				// as deadlines have passed and
				// no one participate or no one revealed
				// creator gets bounty back
				T::Balances::transfer(
					&Self::account_id(&cycle_id.clone()),
					&who,
					cycle.bounty,
					false,
				)?;
				Self::deposit_event(Event::<T>::CycleFailed { cycle_id, creator: who });
				Ok(())
			} else {
				Self::deposit_event(Event::<T>::CycleCompleted {
					cycle_id,
					creator: who,
					random_number: cycle.random_number,
				});
				Ok(())
			}
			//TODO: In above both case any leftover balance from cycle's account should be
			//transferred to org's account
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn account_id(cycle_id: &T::CycleId) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(cycle_id)
		}
	}
}
