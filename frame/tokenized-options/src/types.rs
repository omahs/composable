use crate::Config;
use frame_support::pallet_prelude::*;

use sp_std::fmt::Debug;

// ----------------------------------------------------------------------------------------------------
//		Enums
// ----------------------------------------------------------------------------------------------------
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

#[derive(Clone, Copy, Encode, Decode, Debug, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum WindowType {
	Deposit,
	Purchase,
	Exercise,
	Withdraw,
	End,
}

// ----------------------------------------------------------------------------------------------------
//		Structs and implementations
// ----------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Encode, Decode, Debug, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Epoch<Moment> {
	pub deposit: Moment,
	pub purchase: Moment,
	pub exercise: Moment,
	pub withdraw: Moment,
	pub end: Moment,
}

impl<Moment: Ord> Epoch<Moment> {
	pub fn window_type(&self, moment: Moment) -> Option<WindowType> {
		if moment < self.deposit {
			None
		} else if moment < self.purchase {
			Some(WindowType::Deposit)
		} else if moment < self.exercise {
			Some(WindowType::Purchase)
		} else if moment < self.withdraw {
			Some(WindowType::Exercise)
		} else if moment < self.end {
			Some(WindowType::Withdraw)
		} else {
			Some(WindowType::End)
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct OptionToken<T: Config> {
	// Core attributes of an option, used to uniquely identify an option. quote_asset_id and its price will be added
	pub base_asset_id: T::MayBeAssetId,
	pub base_asset_strike_price: T::Balance,
	pub option_type: OptionType,
	pub expiring_date: T::Moment,

	// Helper attributes
	pub exercise_type: ExerciseType, // Add to core contributes when American is implemented
	pub base_asset_amount_per_option: T::Balance,
	pub quote_asset_id: T::MayBeAssetId,
	pub total_issuance_seller: T::Balance,
	pub total_issuance_buyer: T::Balance,
	pub epoch: Epoch<T::Moment>,
	// pub quote_asset_amount_per_option: Balance, // Assume stablecoin as quote asset right now, so always 1
	// pub quote_asset_strike_price: Balance, // Assume stablecoin as quote asset right now, so always 1
}

#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
pub struct OptionConfig<AssetId, Balance, Moment> {
	pub base_asset_id: AssetId,
	pub base_asset_strike_price: Balance,
	pub option_type: OptionType,
	pub expiring_date: Moment,
	pub quote_asset_id: AssetId,
	pub exercise_type: ExerciseType,
	pub base_asset_amount_per_option: Balance,
	pub total_issuance_seller: Balance,
	pub total_issuance_buyer: Balance,
	pub epoch: Epoch<Moment>,
}

#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct SellerPosition<T: Config> {
	pub option_amount: T::Balance,
	pub shares_amount: T::Balance,
}

impl<T: Config> Default for SellerPosition<T> {
	fn default() -> Self {
		SellerPosition {
			option_amount: T::Balance::default(),
			shares_amount: T::Balance::default(),
		}
	}
}
