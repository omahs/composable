use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weights for pallet_tokenized_options using the Substrate node and recommended hardware.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> tokenized_options::WeightInfo for WeightInfo<T> {
	// Storage: TokenizedOptions AssetToVault (r:1 w:1)
	// Storage: Vault VaultCount (r:1 w:1)
	// Storage: Factory CurrencyCounter (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Vault LpTokensToVaults (r:0 w:1)
	// Storage: Vault Allocations (r:0 w:1)
	// Storage: Vault Vaults (r:0 w:1)
	fn create_asset_vault() -> Weight {
		(144_989_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	// Storage: TokenizedOptions OptionIdToOption (r:0 w:1)
	// Storage: TokenizedOptions OptionHashToOptionId (r:0 w:1)
	// Storage: Factory CurrencyCounter (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: Vault LpTokensToVaults (r:0 w:1)
	// Storage: Vault Allocations (r:0 w:1)
	// Storage: Vault Vaults (r:0 w:1)
	fn create_option() -> Weight {
		(144_989_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	fn sell_option() -> Weight {
		(144_989_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	fn delete_sell_option() -> Weight {
		(144_989_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	fn buy_option() -> Weight {
		(144_989_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	fn exercise_option() -> Weight {
		(144_989_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}

	fn withdraw_collateral() -> Weight {
		(144_989_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
}
