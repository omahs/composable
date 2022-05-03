use frame_support::pallet_prelude::*;
#[allow(unused_variables)]

pub trait TokenizedOptions {
	type AccountId;
	type Balance;
	type AssetId;
	type OptionToken;

	// fn create_option(
	// 	_from: Self::AccountId,
	// 	_option: &OptionToken<Self::AssetId, Self::Balance, Self::UnixTime, Self::Epoch>,
	// ) -> Result<Self::AssetId, DispatchError>;

	fn create_option(
		from: Self::AccountId,
		option: &Self::OptionToken,
	) -> Result<Self::AssetId, DispatchError>;

	fn sell_option(
		from: &Self::AccountId,
		amount: Self::Balance,
		option_id: Self::AssetId,
	) -> Result<(), DispatchError>;

	fn buy_option(
		from: Self::AccountId,
		amount: Self::Balance,
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
