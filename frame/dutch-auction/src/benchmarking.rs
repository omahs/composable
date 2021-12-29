use super::*;
use composable_traits::defi::{DeFiComposableConfig, Sell};
use frame_benchmarking::{
    benchmarks, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::ensure;
use frame_system::RawOrigin;
use sp_std::prelude::*;
use crate::Pallet as DutchAuction;
use codec::{Decode, Encode};
use composable_traits::{currency::{CurrencyFactory}};

// meaningless sell of 1 to 1
pub fn sell_identity<T:Config>() -> Sell<<T as DeFiComposableConfig>::AssetId,<T as DeFiComposableConfig>::Balance> {
	todo!()
    // let one: <T as DeFiComposableConfig>::Balance = 1u64.into();
	// let asset_id = <pallet_currency_factory::Pallet::<T> as  CurrencyFactory<<T as pallet_currency_factory::Config>::DynamicCurrencyId>> ::create().unwrap();
    // Sell::new(asset_id, asset_id, one, one)
}


pub type AssetIdOf<T> =
<T as DeFiComposableConfig>::AssetId;

fn assets<T>() -> [AssetIdOf<T>; 2]
where
	T: Config,
{
	let a = 0u128.to_be_bytes();
	let b = 1u128.to_be_bytes();
	[AssetIdOf::<T>::decode(&mut &a[..]).unwrap(), AssetIdOf::<T>::decode(&mut &b[..]).unwrap()]
}


benchmarks! {
 ask {
    let sell = sell_identity::<T>();
    let caller = RawOrigin::Signed(whitelisted_caller());
 }: _(
    caller,
    sell,
    <_>::default()
  )
   verify {
 	ensure!(0 == 0, "You forgot to sort!")
 }
 impl_benchmark_test_suite!(DutchAuction, crate::mock::runtime::new_test_externalities(), crate::mock::runtime::Runtime,);
}