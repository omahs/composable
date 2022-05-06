use frame_support::pallet_prelude::*;
#[allow(unused_variables)]

pub trait TokenizedOptions {
	type AccountId;
	type Balance;
	type AssetId;
	type OptionToken;
	type OptionConfig;

	fn create_option(option_config: Self::OptionConfig) -> Result<Self::AssetId, DispatchError>;

	fn sell_option(
		from: &Self::AccountId,
		option_amount: Self::Balance,
		option_id: Self::AssetId,
	) -> Result<(), DispatchError>;

	fn buy_option(
		from: &Self::AccountId,
		option_amount: Self::Balance,
		option: Self::AssetId,
	) -> Result<(), DispatchError>;

	fn option_deposit_start(option: Self::AssetId) -> Result<(), DispatchError> {
		Ok(())
	}

	fn option_purchase_start(option: Self::AssetId) -> Result<(), DispatchError> {
		Ok(())
	}

	fn option_exercise_start(option: Self::AssetId) -> Result<(), DispatchError> {
		Ok(())
	}

	fn option_withdraw_start(option: Self::AssetId) -> Result<(), DispatchError> {
		Ok(())
	}

	fn option_end(option: Self::AssetId) -> Result<(), DispatchError> {
		Ok(())
	}
}
