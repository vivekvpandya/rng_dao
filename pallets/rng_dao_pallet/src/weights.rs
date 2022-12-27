use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

pub trait RngDaoWeightInfo {
	fn create_new_rng_cycle() -> Weight;
	fn send_hash() -> Weight;
	fn reveal_secret() -> Weight;
	fn get_random_number() -> Weight;
}

pub struct RuntimeWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> RngDaoWeightInfo for RuntimeWeight<T> {
	fn create_new_rng_cycle() -> Weight {
		Weight::from_ref_time(10_000)
	}
	fn send_hash() -> Weight {
		Weight::from_ref_time(10_000)
	}
	fn reveal_secret() -> Weight {
		Weight::from_ref_time(10_000)
	}
	fn get_random_number() -> Weight {
		Weight::from_ref_time(10_000)
	}
}

// Used in mock runtime only
impl RngDaoWeightInfo for () {
	fn create_new_rng_cycle() -> Weight {
		Weight::from_ref_time(10_000)
	}
	fn send_hash() -> Weight {
		Weight::from_ref_time(10_000)
	}
	fn reveal_secret() -> Weight {
		Weight::from_ref_time(10_000)
	}
	fn get_random_number() -> Weight {
		Weight::from_ref_time(10_000)
	}
}
