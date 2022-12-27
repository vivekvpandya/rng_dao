use super::*;

#[allow(unused)]
use crate::Pallet as RngDao;
use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::{assert_ok, traits::fungible::Mutate};
use frame_system::RawOrigin;
use sp_runtime::traits::{Get, Hash, Keccak256, One};

fn assert_last_event<T: crate::Config>(generic_event: <T as crate::Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	where_clause { where
		T: pallet_balances::Config
		+ frame_system::Config
		+ crate::Config,
		<T as frame_system::Config>::BlockNumber: From<u32>,
		<T as crate::Config>::Balance: From<u128>,
		<T as pallet_balances::Config>::Balance: From<<T as crate::Config>::Balance>,
		<T as crate::Config>::CycleId: From<u128>
	}

	create_new_rng_cycle {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		let cycle_id : T::CycleId = 0_u128.into();
		let caller: T::AccountId = whitelisted_caller();
		let bounty: <T as crate::Config>::Balance = 1000_u128.into();
		let mint_amount: <T as crate::Config>::Balance = 10000_u128.into();
		assert_ok!(<pallet_balances::Pallet::<T> as Mutate<T::AccountId>>::mint_into(&caller, mint_amount.into()));
	}: _(RawOrigin::Signed(caller.clone()), bounty.clone())
	verify {
		assert_eq!(
			Cycles::<T>::get(cycle_id),
			Some(
				RngCycle {
					creator: caller,
					bounty,
					started: 1_u32.into(),
					generators_count: 0_u8,
					revealed_count: 0_u8,
					random_number: 0_u64,
				}
			));
	}

	send_hash {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		let cycle_id : T::CycleId = 0_u128.into();
		let origin: T::AccountId = account("ALICE", 0_u32, 1_u32);
		whitelist_account!(origin);
		let bounty: <T as crate::Config>::Balance = 1000_u128.into();

		let mint_amount: <T as crate::Config>::Balance = 10000_u128.into();
		assert_ok!(<pallet_balances::Pallet::<T> as Mutate<T::AccountId>>::mint_into(&origin, mint_amount.clone().into()));
		assert_ok!(RngDao::<T>::create_new_rng_cycle(RawOrigin::Signed(origin.clone()).into(), bounty.clone()));
		let caller: T::AccountId = whitelisted_caller();
		let deposit: <T as crate::Config>::Balance = 1000_u128.into();
		assert_ok!(<pallet_balances::Pallet::<T> as Mutate<T::AccountId>>::mint_into(&caller, mint_amount.clone().into()));
		let bytes = 1212_u64.to_le_bytes();
		let hash = Keccak256::hash(&bytes);

	}: _(RawOrigin::Signed(caller.clone()), cycle_id.clone(), hash.clone(), false)
	verify {
		assert_last_event::<T>(crate::Event::<T>::HashReceived {cycle_id, sender: caller,
		hash }.into());
	}

	reveal_secret {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		let cycle_id : T::CycleId = 0_u128.into();
		let origin: T::AccountId = account("ALICE", 0_u32, 1_u32);
		whitelist_account!(origin);
		let bounty: <T as crate::Config>::Balance = 1000_u128.into();
		let mint_amount: <T as crate::Config>::Balance = 10000_u128.into();
		assert_ok!(<pallet_balances::Pallet::<T> as Mutate<T::AccountId>>::mint_into(&origin, mint_amount.clone().into()));
		assert_ok!(RngDao::<T>::create_new_rng_cycle(RawOrigin::Signed(origin.clone()).into(), bounty.clone()));
		let caller: T::AccountId = whitelisted_caller();
		let deposit: <T as crate::Config>::Balance = 1000_u128.into();
		assert_ok!(<pallet_balances::Pallet::<T> as Mutate<T::AccountId>>::mint_into(&caller, mint_amount.clone().into()));
		let secret: u64 = 1212_u64;
		let bytes = secret.to_le_bytes();
		let hash = Keccak256::hash(&bytes);
		assert_ok!(RngDao::<T>::send_hash(RawOrigin::Signed(caller.clone()).into(), cycle_id.clone(), hash, false));
		frame_system::Pallet::<T>::set_block_number(
			<T as frame_system::Config>::BlockNumber::one()
			+ <T as crate::Config>::DelayBeforeBots::get()
			+ <T as crate::Config>::DelayBeforeSecondPhase::get()
			+ <T as frame_system::Config>::BlockNumber::one()
		);

	}: _(RawOrigin::Signed(caller.clone()), cycle_id.clone(), secret, false)
	verify {
		assert_last_event::<T>(crate::Event::<T>::SecretReceived {cycle_id, sender: caller}.into());
	}

	// NOTE: get_random_number does more work when cycle fails as it has to return bounty
	get_random_number {
		frame_system::Pallet::<T>::set_block_number(1_u32.into());
		let cycle_id : T::CycleId = 0_u128.into();
		let caller: T::AccountId = whitelisted_caller();
		let origin: T::AccountId = account("ALICE", 0_u32, 1_u32);
		whitelist_account!(origin);
		let bounty: <T as crate::Config>::Balance = 1000_u128.into();
		let mint_amount: <T as crate::Config>::Balance = 10000_u128.into();
		assert_ok!(<pallet_balances::Pallet::<T> as Mutate<T::AccountId>>::mint_into(&caller, mint_amount.clone().into()));
		assert_ok!(RngDao::<T>::create_new_rng_cycle(RawOrigin::Signed(caller.clone()).into(), bounty.clone()));
		let deposit: <T as crate::Config>::Balance = 1000_u128.into();
		assert_ok!(<pallet_balances::Pallet::<T> as Mutate<T::AccountId>>::mint_into(&origin, mint_amount.clone().into()));
		let secret: u64 = 1212_u64;
		let bytes = secret.to_le_bytes();
		let hash = Keccak256::hash(&bytes);
		assert_ok!(RngDao::<T>::send_hash(RawOrigin::Signed(origin.clone()).into(), cycle_id.clone(), hash, false));
		// not revealing the secret so that cycle fails, and bounty will be returned
		frame_system::Pallet::<T>::set_block_number(
			<T as frame_system::Config>::BlockNumber::one()
			+ <T as crate::Config>::DelayBeforeBots::get()
			+ <T as crate::Config>::DelayBeforeSecondPhase::get()
			+ <T as crate::Config>::SecondPhaseDuration::get()
			+ <T as frame_system::Config>::BlockNumber::one()
		);

	}: _(RawOrigin::Signed(caller.clone()), cycle_id.clone())
	verify {
		assert_last_event::<T>(crate::Event::<T>::CycleFailed {cycle_id, creator: caller}.into());
	}

	impl_benchmark_test_suite!(RngDao, crate::mock::ExtBuilder::default().build(), crate::mock::Test);
}
