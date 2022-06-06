use frame_support::parameter_types;
use primitives::currency::ValidCurrency;

pub type CurrencyId = u128;

pub const PICA: CurrencyId = 0;
pub const BTC: CurrencyId = 1;
pub const USDT: CurrencyId = 2;

parameter_types! {
	pub const NativeAssetId: CurrencyId = 0;
}

pub struct AllValidCurrencyId;
impl ValidCurrency<CurrencyId> for AllValidCurrencyId {

	fn valid_currency_id(_currency_id: CurrencyId) -> bool {
			// all other assets in mock are valid
			true
    }
}
