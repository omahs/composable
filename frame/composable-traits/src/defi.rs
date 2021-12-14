//! Common codes for defi pallets

use crate::currency::{AssetIdLike, BalanceLike};
pub trait DeFiComposableConfig: frame_system::Config {
    /// The asset ID type
	type AssetId: AssetIdLike;
	/// The balance type of an account
	type Balance : BalanceLike;
}
