//!  Cross chain specific traits and interfaces


/// generic transaction which can target any pallet and any method in any parachain (local or
/// remote)
/// so it must be encoded in format with widest possible values to incoroporate some chains we do
/// now (similar on how XCM is modelled)
#[derive(Encode)]
pub struct XcmLiquidation<AssetId> {
	pallet: u8,
	method: u8,
	order: Sell<AssetId, u128>,
	strategy: Vec<u128>,
}

impl<AssetId> XcmLiquidation<AssetId> {
	pub fn new(pallet: u8, method: u8, order: Sell<AssetId, u128>, strategy: Vec<u128>) -> Self {
		Self { pallet, method, order, strategy }
	}
}
