use codec::{Codec, FullCodec};
use frame_support::{pallet_prelude::*, sp_std::fmt::Debug};
use scale_info::TypeInfo;

#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo)]
pub enum OptionType {
	Call,
	Put,
}

#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo)]
pub enum ExerciseType {
	European,
	American,
}

pub trait TokenizedOptions {
	type AccountId;
	type AssetId;
	type Balance;

	fn mint_option(
		account: Self::AccountId,
		asset: Self::AssetId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;

	fn exercise_option(
		account: Self::AccountId,
		asset: Self::AssetId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;
}
