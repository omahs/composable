// use core::marker::PhantomData;

// use frame_support::{
// 	dispatch::DispatchResult,
// 	traits::{fungibles, Currency, ExistenceRequirement, Get, SignedImbalance, WithdrawReasons},
// };
// use orml_traits::MultiCurrency;
// use sp_runtime::DispatchError;

// pub struct CurrencyAdapter<MultiCurrencyImpl, ThisAssetId> {
// 	_marker: PhantomData<(MultiCurrencyImpl, ThisAssetId)>,
// }

// impl<
// 		MultiCurrencyImpl: MultiCurrency<AccountId> + fungibles::Mutate<AccountId>,
// 		AccountId,
// 		ThisAssetId: Get<MultiCurrencyImpl::CurrencyId>,
// 	> Currency<AccountId> for CurrencyAdapter<MultiCurrencyImpl, ThisAssetId>
// {
// 	type Balance = <MultiCurrencyImpl as MultiCurrency<AccountId>>::Balance;

// 	type PositiveImbalance = ();

// 	type NegativeImbalance = ();

// 	fn total_balance(who: &AccountId) -> Self::Balance {
// 		MultiCurrencyImpl::total_balance(ThisAssetId::get(), who)
// 	}

// 	fn can_slash(who: &AccountId, value: Self::Balance) -> bool {
// 		MultiCurrencyImpl::can_slash(ThisAssetId::get(), who, value)
// 	}

// 	fn total_issuance() -> Self::Balance {
// 		<MultiCurrencyImpl as MultiCurrency<AccountId>>::total_issuance(ThisAssetId::get())
// 	}

// 	fn minimum_balance() -> Self::Balance {
// 		<MultiCurrencyImpl as MultiCurrency<AccountId>>::minimum_balance(ThisAssetId::get())
// 	}

// 	fn burn(amount: Self::Balance) -> Self::PositiveImbalance {
// 		MultiCurrencyImpl::burn_from(ThisAssetId::get(), amount)
// 	}

// 	fn issue(amount: Self::Balance) -> Self::NegativeImbalance {
// 		MultiCurrencyImpl::issue(ThisAssetId::get(), amount)
// 	}

// 	fn free_balance(who: &AccountId) -> Self::Balance {
// 		<MultiCurrencyImpl as MultiCurrency<AccountId>>::free_balance(ThisAssetId::get(), who)
// 	}

// 	fn ensure_can_withdraw(
// 		who: &AccountId,
// 		amount: Self::Balance,
// 		reasons: WithdrawReasons,
// 		new_balance: Self::Balance,
// 	) -> DispatchResult {
// 		MultiCurrencyImpl::ensure_can_withdraw(ThisAssetId::get(), who, amount)
// 	}

// 	fn transfer(
// 		source: &AccountId,
// 		dest: &AccountId,
// 		value: Self::Balance,
// 		existence_requirement: ExistenceRequirement,
// 	) -> DispatchResult {
// 		MultiCurrencyImpl::ensure_can_withdraw(ThisAssetId::get(), source, value)
// 	}

// 	fn slash(who: &AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
// 		((), MultiCurrencyImpl::slash(ThisAssetId::get(), who, value))
// 	}

// 	fn deposit_into_existing(
// 		who: &AccountId,
// 		value: Self::Balance,
// 	) -> Result<Self::PositiveImbalance, DispatchError> {
// 		MultiCurrencyImpl::deposit(ThisAssetId::get(), who, value)
// 	}

// 	fn deposit_creating(who: &AccountId, value: Self::Balance) -> Self::PositiveImbalance {
// 		MultiCurrencyImpl::deposit(ThisAssetId::get(), who, value)
// 	}

// 	fn withdraw(
// 		who: &AccountId,
// 		value: Self::Balance,
// 		reasons: WithdrawReasons,
// 		liveness: ExistenceRequirement,
// 	) -> Result<Self::NegativeImbalance, DispatchError> {
// 		MultiCurrencyImpl::withdraw(ThisAssetId::get(), who, value)
// 	}

// 	fn make_free_balance_be(
// 		who: &AccountId,
// 		balance: Self::Balance,
// 	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
// 	}
// }
