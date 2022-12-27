use crate::{mock::*, Error, Event, RngCycle};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::{Hash, Keccak256};

#[test]
fn create_new_rng_cycle_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		assert_eq!(
			RngDao::cycles(0_u128),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&0_u128)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
	});
}

#[test]
fn create_new_rng_cycle_fails_due_to_low_bounty() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), 50),
			Error::<Test>::BountyMustBeGreaterThanMinBounty
		);
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
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());

		// BOB takes part
		let bob_secret = 9897_u64;
		let bob_hash = Keccak256::hash(&bob_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), cycle_id, bob_hash, false));
		System::assert_last_event(
			Event::HashReceived { cycle_id, sender: BOB, hash: bob_hash }.into(),
		);
		// CHARLIE takes part
		let charlie_secret = 120019_u64;
		let charlie_hash = Keccak256::hash(&charlie_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(
			RuntimeOrigin::signed(CHARLIE),
			cycle_id,
			charlie_hash,
			false
		));
		System::assert_last_event(
			Event::HashReceived { cycle_id, sender: CHARLIE, hash: charlie_hash }.into(),
		);

		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 1);

		// BOT takes part
		let bot_secret = 807_u64;
		let bot_hash = Keccak256::hash(&bot_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOT), cycle_id, bot_hash, true));
		System::assert_last_event(
			Event::HashReceived { cycle_id, sender: BOT, hash: bot_hash }.into(),
		);

		let expected_random_number = 0_u64 ^ bob_secret ^ charlie_secret ^ bot_secret;

		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 1);

		// BOB reveals
		assert_ok!(RngDao::reveal_secret(RuntimeOrigin::signed(BOB), cycle_id, bob_secret, false));
		System::assert_last_event(Event::SecretReceived { cycle_id, sender: BOB }.into());
		// CHARLIE reveals
		assert_ok!(RngDao::reveal_secret(
			RuntimeOrigin::signed(CHARLIE),
			cycle_id,
			charlie_secret,
			false
		));
		System::assert_last_event(Event::SecretReceived { cycle_id, sender: CHARLIE }.into());
		// BOT reveals
		assert_ok!(RngDao::reveal_secret(RuntimeOrigin::signed(BOT), cycle_id, bot_secret, true));
		System::assert_last_event(Event::SecretReceived { cycle_id, sender: BOT }.into());

		System::set_block_number(
			1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 5
			/*SecondPhaseDuration*/ + 1,
		);

		assert_ok!(RngDao::get_random_number(RuntimeOrigin::signed(ALICE), cycle_id));
		System::assert_last_event(
			Event::CycleCompleted {
				cycle_id,
				creator: ALICE,
				random_number: expected_random_number,
			}
			.into(),
		);

		assert_eq!(
			RngDao::cycles(cycle_id),
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

#[test]
fn second_phase_not_started_yet_error() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		let bob_secret = 807_u64;
		let bob_hash = Keccak256::hash(&bob_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), cycle_id, bob_hash, false));
		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 1);
		assert_noop!(
			RngDao::reveal_secret(RuntimeOrigin::signed(BOB), cycle_id, 10_u64, false),
			Error::<Test>::SecretDoesNotMatchHash
		);
	});
}

#[test]
fn not_submited_hash_in_first_phase_error() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		let bob_secret = 807_u64;
		let bob_hash = Keccak256::hash(&bob_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), cycle_id, bob_hash, false));
		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 1);
		assert_noop!(
			RngDao::reveal_secret(RuntimeOrigin::signed(EVE), cycle_id, 10_u64, false),
			Error::<Test>::NotSubmitedHashInFirstPhase
		);
	});
}

#[test]
fn random_number_not_yet_generated_error() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		let bob_secret = 807_u64;
		let bob_hash = Keccak256::hash(&bob_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), cycle_id, bob_hash, false));
		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 1);
		assert_ok!(RngDao::reveal_secret(RuntimeOrigin::signed(BOB), cycle_id, bob_secret, false));
		assert_noop!(
			RngDao::get_random_number(RuntimeOrigin::signed(ALICE), 0_u128),
			Error::<Test>::RandomNumberNotYetGenerated
		);
	});
}

#[test]
fn return_bounty_case1() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		// no generators participated and deadline passed
		System::set_block_number(
			1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 5
			/*SecondPhaseDuration*/ + 1,
		);
		assert_ok!(RngDao::get_random_number(RuntimeOrigin::signed(ALICE), cycle_id));
		System::assert_last_event(Event::CycleFailed { cycle_id, creator: ALICE }.into());
		// ALICE get's her bounty value back
		assert_eq!(Balances::free_balance(ALICE), free_balance);
	});
}

#[test]
fn return_bounty_case2() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		let bob_secret = 807_u64;
		let bob_hash = Keccak256::hash(&bob_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), cycle_id, bob_hash, false));
		// no generators revealed correct secret in time
		System::set_block_number(
			1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 5
			/*SecondPhaseDuration*/ + 1,
		);
		assert_ok!(RngDao::get_random_number(RuntimeOrigin::signed(ALICE), cycle_id));
		System::assert_last_event(Event::CycleFailed { cycle_id, creator: ALICE }.into());
		// ALICE get's her bounty value back
		assert_eq!(Balances::free_balance(ALICE), free_balance);
	});
}

#[test]
fn not_authorized_to_get_random_number_error() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		let bob_secret = 807_u64;
		let bob_hash = Keccak256::hash(&bob_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), cycle_id, bob_hash, false));
		System::set_block_number(1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 1);
		assert_ok!(RngDao::reveal_secret(RuntimeOrigin::signed(BOB), cycle_id, bob_secret, false));
		System::set_block_number(
			1 + 3 /*DelayBeforeBots*/ + 2 /*DelayBeforeSecondPhase*/ + 5
			/*SecondPhaseDuration*/ + 1,
		);
		assert_noop!(
			RngDao::get_random_number(RuntimeOrigin::signed(BOB), cycle_id),
			Error::<Test>::NotAuthorizedToGetRandomNumber
		);
	});
}

#[test]
fn secret_does_not_match_hash_error() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		let bob_secret = 807_u64;
		let bob_hash = Keccak256::hash(&bob_secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), cycle_id, bob_hash, false));
		assert_noop!(
			RngDao::reveal_secret(RuntimeOrigin::signed(BOB), cycle_id, bob_secret, false),
			Error::<Test>::SecondPhaseNotStartedYet
		);
	});
}

#[test]
fn bots_cannot_participate_before_delay() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		let bot_secret = 807_u64;
		let bot_hash = Keccak256::hash(&bot_secret.to_le_bytes());
		assert_noop!(
			RngDao::send_hash(RuntimeOrigin::signed(BOT), cycle_id, bot_hash, true),
			Error::<Test>::BotsNotAllowedYet
		);
	});
}

#[test]
fn max_generators_error() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		let bounty = 150;
		let free_balance = Balances::free_balance(ALICE);
		assert_ok!(RngDao::create_new_rng_cycle(RuntimeOrigin::signed(ALICE), bounty));
		assert_eq!(RngDao::get_cycle_count(), 1_u128);
		let cycle_id = 0_u128;
		assert_eq!(
			RngDao::cycles(cycle_id),
			Some(RngCycle {
				creator: ALICE,
				bounty,
				started: 1,
				generators_count: 0_u8,
				revealed_count: 0_u8,
				random_number: 0_u64,
			})
		);
		assert_eq!(Balances::free_balance(ALICE), free_balance - bounty);
		assert_eq!(Balances::free_balance(RngDao::account_id(&cycle_id)), bounty);
		System::assert_last_event(Event::CycleCreated { bounty, creator: ALICE }.into());
		let secret = 807_u64;
		let hash = Keccak256::hash(&secret.to_le_bytes());
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(BOB), cycle_id, hash, false));
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(CHARLIE), cycle_id, hash, false));
		assert_ok!(RngDao::send_hash(RuntimeOrigin::signed(EVE), cycle_id, hash, false));
		assert_noop!(
			RngDao::send_hash(RuntimeOrigin::signed(TOM), cycle_id, hash, false),
			Error::<Test>::MaxGeneratorsReached
		);
	});
}
