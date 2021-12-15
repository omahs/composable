//! Common codes for defi pallets

use codec::{Encode, Decode};
use scale_info::TypeInfo;

use crate::currency::{AssetIdLike, BalanceLike};
pub trait DeFiComposableConfig: frame_system::Config {
    /// The asset ID type
	type AssetId: AssetIdLike;
	/// The balance type of an account
	type Balance : BalanceLike;
}

/// take `quote` currency and give `base` currency
#[derive(Encode, Decode, TypeInfo)]
pub struct Sell<AssetId, Balance> {
	pub pair: CurrencyPair<AssetId>,
	/// minimal amount of `quote` for given unit of `base` 
	pub limit: Balance,
}

/// given `base`, how much `quote` needed for unit
/// see [currency pair](https://www.investopedia.com/terms/c/currencypair.asp)
#[derive(Encode, Decode, TypeInfo)]
pub struct CurrencyPair<AssetId> {
	/// See [Base Currency](https://www.investopedia.com/terms/b/basecurrency.asp)
	pub base: AssetId,
	/// counter currency
	pub quote: AssetId,
}

/// type parameters for traits in pure defi area
pub trait DeFiTrait {
	/// The asset ID type
	type AssetId: AssetIdLike;
	/// The balance type of an account
	type Balance : BalanceLike;
	/// The user account identifier type for the runtime	
	type AccountId;
}