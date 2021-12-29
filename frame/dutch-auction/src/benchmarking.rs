use super::*;
use composable_traits::defi::{DeFiComposableConfig, Sell, CurrencyPair, Take};
use frame_benchmarking::{
    benchmarks, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::{ensure, traits::fungibles::Mutate};
use frame_system::RawOrigin;
use sp_std::prelude::*;
use crate::Pallet as DutchAuction;
use codec::{Decode, Encode};
use composable_traits::{currency::{CurrencyFactory}};

// meaningless sell of 1 to 1
pub fn sell_identity<T:Config>() -> Sell<<T as DeFiComposableConfig>::AssetId,<T as DeFiComposableConfig>::Balance> {
    let one: <T as DeFiComposableConfig>::Balance = 1u64.into();
	let pair = assets::<T>();
    Sell::new(pair.base, pair.quote, one, one)
}

// meaningless take of 1 to 1
pub fn take_identity<T:Config>() -> Take<<T as DeFiComposableConfig>::Balance> {
    let one: <T as DeFiComposableConfig>::Balance = 1u64.into();
	Take::new(one, one)
}


pub type AssetIdOf<T> =
<T as DeFiComposableConfig>::AssetId;

fn assets<T>() -> CurrencyPair<AssetIdOf<T>>
where
	T: Config,
{
	let a = 0u128.to_be_bytes();
	let b = 1u128.to_be_bytes();
	CurrencyPair::new(AssetIdOf::<T>::decode(&mut &a[..]).unwrap(), AssetIdOf::<T>::decode(&mut &b[..]).unwrap())
}


benchmarks! {
    where_clause {
        where 
        <T as Config>::MultiCurrency: 
    			Mutate<T::AccountId, Balance = T::Balance, AssetId = T::AssetId>, 
    }
    ask {
        let sell = sell_identity::<T>();
        let account_id : T::AccountId = whitelisted_caller();
        let caller = RawOrigin::Signed(account_id.clone());
        let amount: T::Balance = 1_000_000u64.into();
        <T as pallet::Config>::MultiCurrency::mint_into(sell.pair.base, &account_id, amount).unwrap();
        }: _(
            caller,
            sell,
            <_>::default()
        )
    take {
        let sell = sell_identity::<T>();
        let account_id : T::AccountId = whitelisted_caller();
        let caller = RawOrigin::Signed(account_id.clone());
        let amount: T::Balance = 1_000_000u64.into();
        let order_id = OrdersIndex::<T>::get();
        <T as pallet::Config>::MultiCurrency::mint_into(sell.pair.base, &account_id, amount).unwrap();
        <T as pallet::Config>::MultiCurrency::mint_into(sell.pair.quote, &account_id, amount).unwrap();
        DutchAuction::<T>::ask(caller.clone().into(), sell, <_>::default()).unwrap();
        let take_order = take_identity::<T>();
        }: _(
            caller,
            order_id,
            take_order
        )      
    liqudate {
        let sell = sell_identity::<T>();
        let account_id : T::AccountId = whitelisted_caller();
        let caller = RawOrigin::Signed(account_id.clone());
        let amount: T::Balance = 1_000_000u64.into();
        <T as pallet::Config>::MultiCurrency::mint_into(sell.pair.base, &account_id, amount).unwrap();
        }: _(
            caller,
            sell,
            <_>::default()
        )          
}

impl_benchmark_test_suite!(DutchAuction, crate::mock::runtime::new_test_externalities(), crate::mock::runtime::Runtime,);