use primitives::currency::CurrencyId;

#[allow(non_snake_case)]
pub const fn AssetId(id: u128) -> AssetId {
	CurrencyId(id)
}

pub type AssetId = CurrencyId;
pub const BTC: AssetId = AssetId(2000);
pub const DOT: AssetId = AssetId(4000);
pub const PICA: AssetId = CurrencyId::PICA;
pub const LAYR: AssetId = CurrencyId::LAYR;
pub const USDC: AssetId = CurrencyId::USDC;
pub const ETH: AssetId = AssetId(5000);

pub const ASSETS: [AssetId; 4] = [BTC, DOT, PICA, LAYR];
pub const ASSETS_WITH_USDC: [AssetId; 5] = [BTC, DOT, PICA, LAYR, USDC];
