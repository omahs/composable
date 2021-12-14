use sp_runtime::{DispatchError, Permill};

use crate::dex::LimitOrderbook;

pub trait Liquidate {
	type AssetId;
	type Balance;
	type AccountId;
	type LiquidationId;

	fn initiate_liquidation(
		source_account: &Self::AccountId,
		source_asset_id: Self::AssetId,
		source_asset_price: Self::Balance,
		target_asset_id: Self::AssetId,
		target_account: &Self::AccountId,
		total_amount: Self::Balance,
	) -> Result<Self::LiquidationId, DispatchError>;
}

impl<T: Orderbook> Liquidate for T {
	type AssetId = <Self as LimitOrderbook>::AssetId;
	type Balance = <Self as LimitOrderbook>::Balance;
	type AccountId = <Self as LimitOrderbook>::AccountId;
	type LiquidationId = <Self as LimitOrderbook>::OrderId;

	fn initiate_liquidation(
		source_account: &Self::AccountId,
		source_asset_id: Self::AssetId,
		_source_asset_price: Self::Balance,
		target_asset_id: Self::AssetId,
		_target_account: &Self::AccountId,
		total_amount: Self::Balance,
	) -> Result<Self::LiquidationId, DispatchError> {
		<T as LimitOrderbook>::ask(
			source_account,
			source_asset_id,
			target_asset_id,
			total_amount,
			Permill::from_perthousand(0),
		)
	}
}
