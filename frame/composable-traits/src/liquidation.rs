use sp_runtime::{DispatchError, Permill};

use crate::{dex::{LimitOrderbook,}, defi::{SellTrait, DeFiTrait}, auction::AuctionStepFunction};

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
