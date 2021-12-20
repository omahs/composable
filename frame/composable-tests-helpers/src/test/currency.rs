#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct CurrencyId(u128);



#[derive(
	PartialOrd,
	Ord,
	PartialEq,
	Eq,
	Debug,
	Copy,
	Clone,
	codec::Encode,
	codec::Decode,
	serde::Serialize,
	serde::Deserialize,
	TypeInfo,
)]
pub enum MockCurrencyId {
	PICA,
	BTC,
	ETH,
	LTC,
	USDT,
	LpToken(u128),
}

impl From<u128> for MockCurrencyId {
	fn from(id: u128) -> Self {
		match id {
			0 => MockCurrencyId::PICA,
			1 => MockCurrencyId::BTC,
			2 => MockCurrencyId::ETH,
			3 => MockCurrencyId::LTC,
			4 => MockCurrencyId::USDT,
			5 => MockCurrencyId::LpToken(0),
			_ => unreachable!(),
		}
	}
}

impl Default for MockCurrencyId {
	fn default() -> Self {
		MockCurrencyId::PICA
	}
}

impl PriceableAsset for MockCurrencyId {
	fn smallest_unit_exponent(self) -> composable_traits::currency::Exponent {
		match self {
			MockCurrencyId::PICA => 0,
			MockCurrencyId::BTC => 8,
			MockCurrencyId::ETH => 18,
			MockCurrencyId::LTC => 8,
			MockCurrencyId::USDT => 2,
			MockCurrencyId::LpToken(_) => 0,
		}
	}
}

impl DynamicCurrencyId for MockCurrencyId {
	fn next(self) -> Result<Self, DispatchError> {
		match self {
			MockCurrencyId::LpToken(x) => Ok(MockCurrencyId::LpToken(
				x.checked_add(1).ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?,
			)),
			_ => unreachable!(),
		}
	}
}




pub struct Pica {

}
