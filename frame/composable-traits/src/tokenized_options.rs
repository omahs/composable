use frame_support::{pallet_prelude::*, sp_std::fmt::Debug, traits::UnixTime};
use scale_info::TypeInfo;

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
		_from: Self::AccountId,
		_option: &Self::OptionToken,
	) -> Result<Self::AssetId, DispatchError>;

	fn sell_option(
		_from: &Self::AccountId,
		_amount: Self::Balance,
		_option_id: Self::AssetId,
	) -> Result<(), DispatchError>;

	fn buy_option(
		_from: Self::AccountId,
		_amount: Self::Balance,
		_option: Self::AssetId,
	) -> Result<(), DispatchError>;
}
