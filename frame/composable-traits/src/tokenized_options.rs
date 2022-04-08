use frame_support::{pallet_prelude::*, sp_std::fmt::Debug};
use scale_info::TypeInfo;

#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum OptionType {
	Call,
	Put,
}

#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum ExerciseType {
	European,
	American,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct OptionToken<AssetId, Balance> {
	pub base_asset_id: AssetId,
	pub strike_price: Balance,
	pub option_type: OptionType,
	pub exercise_type: ExerciseType,
}

pub trait TokenizedOptions {
	type AccountId;
	type Balance;
	type AssetId;
	type VaultId;

	fn create_option(
		_from: Self::AccountId,
		_option: &OptionToken<Self::AssetId, Self::Balance>,
	) -> Result<Self::AssetId, DispatchError>;

	fn create_option_with_vault(
		_from: Self::AccountId,
		_option: &OptionToken<Self::AssetId, Self::Balance>,
	) -> Result<(Self::AssetId, Self::VaultId), DispatchError>;

	fn sell_option(
		_from: Self::AccountId,
		_amount: Self::Balance,
		_option_id: Self::AssetId,
	) -> Result<(), DispatchError>;

	fn buy_option(
		_from: Self::AccountId,
		_amount: Self::Balance,
		_option: Self::AssetId,
	) -> Result<(), DispatchError>;
}
