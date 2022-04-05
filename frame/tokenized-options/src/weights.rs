use frame_support::weights::Weight;
use sp_std::marker::PhantomData;

const WEIGHT: i32 = 0;

pub trait WeightInfo {
	fn create_asset_vault() -> Weight;
	fn create_option_vault() -> Weight;
	fn sell_option() -> Weight;
	fn buy_option() -> Weight;
}

/// Weights for pallet_tokenized_options using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn create_asset_vault() -> Weight {
		WEIGHT as Weight
	}

	fn create_option_vault() -> Weight {
		WEIGHT as Weight
	}

	fn sell_option() -> Weight {
		WEIGHT as Weight
	}

	fn buy_option() -> Weight {
		WEIGHT as Weight
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_asset_vault() -> Weight {
		WEIGHT as Weight
	}

	fn create_option_vault() -> Weight {
		WEIGHT as Weight
	}

	fn sell_option() -> Weight {
		WEIGHT as Weight
	}

	fn buy_option() -> Weight {
		WEIGHT as Weight
	}
}
