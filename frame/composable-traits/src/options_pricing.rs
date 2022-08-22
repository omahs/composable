use frame_support::pallet_prelude::*;
use crate::tokenized_options::OptionType;
#[allow(unused_variables)]
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
pub struct BlackScholesParams<AssetId, Balance, Moment> {
	pub base_asset_id: AssetId,
	pub base_asset_strike_price: Balance,
	pub base_asset_spot_price: Balance,
	pub expiring_date: Moment,
	pub option_type: OptionType,
	pub total_issuance_buyer: Balance,
	pub total_premium_paid: Balance,
}

pub trait OptionsPricing {
	type AssetId;
	type Balance;
	type Moment;

	fn calculate_option_price(
		params: BlackScholesParams<Self::AssetId, Self::Balance, Self::Moment>,
	) -> Result<Self::Balance, DispatchError>;

	fn calculate_option_greeks(
		params: BlackScholesParams<Self::AssetId, Self::Balance, Self::Moment>,
	) -> Result<(), DispatchError>;
}
