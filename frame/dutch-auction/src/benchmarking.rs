use super::*;
use composable_traits::defi::{DeFiComposableConfig, Sell};
use frame_benchmarking::{
    benchmarks, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_support::ensure;
use frame_system::RawOrigin;
use sp_std::prelude::*;
use crate::Pallet as DutchAuction;

use composable_traits::{currency::{CurrencyFactory}};

// meaningless sell of 1 to 1
pub fn sell_identity<T:Config + pallet_currency_factory::Config>() -> Sell<<T as DeFiComposableConfig>::AssetId,<T as DeFiComposableConfig>::Balance> {
	todo!()
    // let one: <T as DeFiComposableConfig>::Balance = 1u64.into();
	// let asset_id = <pallet_currency_factory::Pallet::<T> as  CurrencyFactory<<T as pallet_currency_factory::Config>::DynamicCurrencyId>> ::create().unwrap();
    // Sell::new(asset_id, asset_id, one, one)
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
}

// pub fn new_test_externalities() -> sp_io::TestExternalities {
// 	let mut storage = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
// 	let balances = vec![
// 		(AccountId::from(ALICE), 1_000_000_000_000_000_000_000_000),
// 		(AccountId::from(BOB), 1_000_000_000_000_000_000_000_000),
// 	];

// 	pallet_balances::GenesisConfig::<Runtime> { balances: balances.clone() }
// 		.assimilate_storage(&mut storage)
// 		.unwrap();

// 	let mut externatlities = sp_io::TestExternalities::new(storage);
// 	externatlities.execute_with(|| {
// 		System::set_block_number(42);
// 		Timestamp::set_timestamp(System::block_number() * MILLISECS_PER_BLOCK);
// 	});
// 	externatlities
// }

//impl_benchmark_test_suite!(DutchAuction, new_test_externalities(), crate::mock::runtime::Runtime,);
impl_benchmark_test_suite!(DutchAuction, crate::mock::new_test_ext(), crate::mock::Test,);
