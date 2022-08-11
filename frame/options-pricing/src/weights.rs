use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

const WEIGHT: i32 = 1_000;

pub trait WeightInfo {
	fn calculate_option_price() -> Weight;
}

/// Weights for pallet_tokenized_options using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn calculate_option_price() -> Weight {
		WEIGHT as Weight
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn calculate_option_price() -> Weight {
		WEIGHT as Weight
	}
}
