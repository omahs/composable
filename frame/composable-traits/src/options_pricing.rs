use frame_support::pallet_prelude::*;
#[allow(unused_variables)]
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
pub struct BlackScholesParams<AssetId, Balance, Moment> {
	pub base_asset_id: AssetId,
	pub base_asset_strike_price: Balance,
	pub base_asset_spot_price: Balance,
	pub expiring_date: Moment,
	pub total_issuance_buyer: Balance,
	pub total_premium_paid: Balance,
}

pub trait OptionsPricing {
	type AssetId;
	type Balance;
	type Moment;
	type OptionId;

	fn calculate_option_price(
		params: BlackScholesParams<Self::AssetId, Self::Balance, Self::Moment>,
	) -> Result<Self::Balance, DispatchError>;
}
