#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
use frame_support::{pallet_prelude::*, RuntimeDebug, ensure, PalletId, traits::fungible::{Inspect, Transfer}};
use frame_system::pallet_prelude::*;
use sp_runtime::{ArithmeticError, traits::{AtLeast32BitUnsigned, AccountIdConversion, CheckedAdd, CheckedSub, CheckedDiv, One, Zero}};
use codec::FullCodec;
use core::fmt::Debug;

#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo)]
pub enum Status {
	Active,
	CompletedWithSuccess,
	CompletedWithFailure,
}

#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo)]
pub struct RngCycle<AccountId, Balance> {
	pub	creator: AccountId,
	pub bounty: Balance,
	pub status: Status,
	/// random_number is only valid if Status is `CompletedWithSuccess`
	pub random_number: u64,
}

	pub(crate) type BalanceOf<T> = <T as Config>::Balance;
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type RngCycleOf<T> = RngCycle<AccountIdOf<T>, BalanceOf<T>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;


		type CycleId: FullCodec + MaxEncodedLen + Default + Debug + TypeInfo +
			CheckedAdd + One + Sized + Zero + Clone + Copy;
		type Balance: FullCodec + MaxEncodedLen + Default + Debug+ TypeInfo + AtLeast32BitUnsigned + CheckedAdd + CheckedSub + CheckedDiv;

		type Balances: Transfer<AccountIdOf<Self>, Balance = BalanceOf<Self>>;

		#[pallet::constant]
		type MinBounty: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CycleCreated {bounty: T::Balance, creator: T::AccountId},
	}

	#[pallet::error]
	pub enum Error<T> {
		BountyMustBeGreaterThanMinBounty,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_new_rng_cycle(origin: OriginFor<T>, bounty: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(bounty >= T::MinBounty::get(), Error::<T>::BountyMustBeGreaterThanMinBounty);
			let cycle_id = CycleCount::<T>::try_mutate(|cycle_count| -> Result<T::CycleId,
				DispatchError> {
				let cycle_id = *cycle_count;

				Cycles::<T>::insert(cycle_id.clone(),
					RngCycleOf::<T> {creator: who.clone(), bounty: bounty.clone(), status: Status::Active, random_number: 0_u64 },
				);
				*cycle_count =
					cycle_id.checked_add(&T::CycleId::one()).ok_or(ArithmeticError::Overflow)?;
				Ok(cycle_id)
				}
			)?;
			//TODO: Move bounty value to a cycle specific account.
			T::Balances::transfer(&who, &Self::account_id(&cycle_id), bounty.clone(), true)?;

			Self::deposit_event(Event::CycleCreated { bounty, creator: who });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn account_id(cycle_id: &T::CycleId) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(cycle_id)
		}
	}
}
