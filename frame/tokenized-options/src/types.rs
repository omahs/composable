use crate::Config;
use frame_support::pallet_prelude::*;

use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash, Zero};
use sp_std::fmt::Debug;

// ----------------------------------------------------------------------------------------------------
//		Enums
// ----------------------------------------------------------------------------------------------------
/// Indicates the type of option: `Call` or `Put`
#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum OptionType {
	Call,
	Put,
}

/// Indicates the type of exercise of the option: `European` or `American`
#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum ExerciseType {
	European,
	American,
}

/// Indicates the type of phases of the option.
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

/// Stores the timestamps of an epoch.
/// An Epoch is divided into 4 phases: deposit, purchase, exercise, withdraw.
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

/// Represent the option with the attributes to be configured
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct OptionToken<T: Config> {
	// Core attributes of an option, used to uniquely identify an option
	pub base_asset_id: T::MayBeAssetId,
	pub quote_asset_id: T::MayBeAssetId,
	pub base_asset_strike_price: T::Balance,
	pub quote_asset_strike_price: T::Balance,
	pub option_type: OptionType,
	pub expiring_date: T::Moment,
	pub exercise_type: ExerciseType,

	// Helper attributes
	pub base_asset_amount_per_option: T::Balance,
	pub quote_asset_amount_per_option: T::Balance,
	pub total_issuance_seller: T::Balance,
	pub total_issuance_buyer: T::Balance,
	pub epoch: Epoch<T::Moment>,
}

impl<T: Config> OptionToken<T> {
	pub fn generate_id(&self) -> H256 {
		BlakeTwo256::hash_of(&(
			self.base_asset_id,
			self.quote_asset_id,
			self.base_asset_strike_price,
			self.quote_asset_strike_price,
			self.option_type,
			self.expiring_date,
			self.exercise_type,
		))
	}
}

/// Configuration for creating an option
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
pub struct OptionConfig<AssetId, Balance, Moment> {
	pub base_asset_id: AssetId,
	pub quote_asset_id: AssetId,
	pub base_asset_strike_price: Balance,
	pub quote_asset_strike_price: Balance,
	pub option_type: OptionType,
	pub expiring_date: Moment,
	pub exercise_type: ExerciseType,
	pub base_asset_amount_per_option: Balance,
	pub quote_asset_amount_per_option: Balance,
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
		SellerPosition { option_amount: T::Balance::zero(), shares_amount: T::Balance::zero() }
	}
}
