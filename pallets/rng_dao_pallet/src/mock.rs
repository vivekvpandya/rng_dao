use crate as pallet_rng_dao;
use crate::pallet::Config;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU32},
	PalletId,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	generic::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
type AccountId = u64;
type CycleId = u128;

pub static ALICE: AccountId = 1;
pub static BOB: AccountId = 2;
pub static CHARLIE: AccountId = 3;
pub static EVE: AccountId = 4;
pub static TOM: AccountId = 5;
pub static BOT: AccountId = 6;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		RngDao: pallet_rng_dao,
		Balances: pallet_balances,
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u32;
	type BlockNumber = u32;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header<u32, BlakeTwo256>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU32<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub MinBounty: u128 = 100_u128;
	pub Deposit: u128 = 300_u128;
	pub TestPalletId : PalletId = PalletId(*b"rng_dao_");
	pub DelayBeforeBots: u32 = 3_u32;
	pub DelayBeforeSecondPhase: u32 = 2_u32;
	pub SecondPhaseDuration: u32 = 5_u32;
	pub MaxGenerators: u8 = 3_u8;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CycleId = CycleId;
	type Balance = Balance;
	type Deposit = Deposit;
	type MinBounty = MinBounty;
	type Balances = Balances;
	type PalletId = TestPalletId;
	type DelayBeforeBots = DelayBeforeBots;
	type DelayBeforeSecondPhase = DelayBeforeSecondPhase;
	type SecondPhaseDuration = SecondPhaseDuration;
	type MaxGenerators = MaxGenerators;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, 1_000),
				(BOB, 1_000),
				(CHARLIE, 1_000),
				(EVE, 1_000),
				(TOM, 1_000),
				(BOT, 1_000),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		t.into()
	}
}
