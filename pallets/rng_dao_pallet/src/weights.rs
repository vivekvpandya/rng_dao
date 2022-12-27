use frame_support::weights::Weight;

pub trait RngDaoWeightInfo {
	fn create_new_rng_cycle() -> Weight;
	fn send_hash() -> Weight;
	fn reveal_secret() -> Weight;
	fn get_random_number() -> Weight;
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
