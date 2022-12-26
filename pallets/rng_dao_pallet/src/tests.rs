use crate::{mock::*, Error, Event, Status, RngCycle};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_new_rng_cycle_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(1_u64);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(1), 150));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		assert_eq!(RngDao::cycles(0_u128),
			Some(RngCycle {
					creator: 1,
					bounty,
					status: Status::Active,
					random_number: 0
				})
		);
		assert_eq!(Balances::free_balance(1_u64), free_balance -  bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&0_u128)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty: 150, creator: 1 }.into());
	});
}

#[test]
fn create_new_rng_cycle_fails_due_to_low_bounty() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(1), 50),
		Error::<Test>::BountyMustBeGreaterThanMinBounty);
		assert_eq!(RngDao::get_cycle_count(), 0_u128);
	});
}
