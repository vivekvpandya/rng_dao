use crate::{mock::*, Error, Event, RngCycle};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::{Keccak256, Hash};

#[test]
fn create_new_rng_cycle_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		assert_eq!(RngDao::cycles(0_u128),
			Some(RngCycle {
					creator: ALICE,
					bounty,
					started: 1,
					generators_count: 0_u8,
					revealed_count: 0_u8,
					random_number: 0_u64,
				})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance -  bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&0_u128)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
	});
}

#[test]
fn create_new_rng_cycle_fails_due_to_low_bounty() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), 50),
		Error::<Test>::BountyMustBeGreaterThanMinBounty);
		assert_eq!(RngDao::get_cycle_count(), 0_u128);
	});
}

#[test]
fn basic_rng_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 200;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		assert_eq!(RngDao::cycles(0_u128),
			Some(RngCycle {
					creator: ALICE,
					bounty,
					started: 1,
					generators_count: 0_u8,
					revealed_count: 0_u8,
					random_number: 0_u64,
				})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance -  bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&0_u128)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());

		// BOB takes part
		let bob_secret =  9897_u64;
		let bob_hash = Keccak256::hash(&bob_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), 0_u128, bob_hash, false));
		// CHARLIE takes part
		let charlie_secret =  120019_u64;
		let charlie_hash = Keccak256::hash(&charlie_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(CHARLIE), 0_u128, charlie_hash, false));

		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 1);

		// BOT takes part
		let bot_secret =  807_u64;
		let bot_hash = Keccak256::hash(&bot_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOT), 0_u128, bot_hash, true));

		let expected_random_number = 0_u64 ^ bob_secret ^ charlie_secret ^ bot_secret;

		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 1);

		// BOB reveals
		assert_ok!(RngDao::reveal_secret(RuntimeOrigin::signed(BOB), 0_u128, bob_secret, false));
		// CHARLIE reveals
		assert_ok!(RngDao::reveal_secret(RuntimeOrigin::signed(CHARLIE), 0_u128, charlie_secret, false));
		// BOT reveals
		assert_ok!(RngDao::reveal_secret(RuntimeOrigin::signed(BOT), 0_u128, bot_secret, true));

		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 5
			/*SecondPhaseDuration*/ + 1 );

		assert_ok!(RngDao::get_random_number(RuntimeOrigin::signed(ALICE), 0_u128));

		assert_eq!(RngDao::cycles(0_u128),
			Some(RngCycle {
					creator: ALICE,
					bounty,
					started: 1,
					generators_count: 3_u8,
					revealed_count: 3_u8,
					random_number: expected_random_number,
				})
		);


	});
}
