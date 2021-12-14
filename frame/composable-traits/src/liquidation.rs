use sp_runtime::{DispatchError, Permill};

use crate::dex::{LimitOrderbook, DeFiTrait, Sell};

pub trait Liquidate {
	type AssetId;
	type Balance;
	type AccountId;
	type LiquidationId;
	type AmmConfiguration;
	
	fn initiate_liquidation(
		source_account: &Self::AccountId,
		source_asset_id: Self::AssetId,
		source_asset_price: Self::Balance,
		target_asset_id: Self::AssetId,
		target_account: &Self::AccountId,
		total_amount: Self::Balance,
	) -> Result<Self::LiquidationId, DispatchError>;
}

impl<T: LimitOrderbook> Liquidate for T {
	type AssetId = <Self as DeFiTrait>::AssetId;
	type Balance = <Self as DeFiTrait>::Balance;
	type AccountId = <Self as DeFiTrait>::AccountId;
	type LiquidationId = <Self as LimitOrderbook>::OrderId;
	type AmmConfiguration = <Self as LimitOrderbook>::AmmConfiguration;
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
			source_account,
		Sell::new(source_asset_id, target_asset_id, _source_asset_price), 
		total_amount,
				<_>::default(),
		)
	}
}
