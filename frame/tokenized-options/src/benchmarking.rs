//! Benchmarks for Template Pallet
// #![cfg(feature = "runtime-benchmarks")]

use crate::*;
use crate::{Pallet as TokenizedOptions} ;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::{EventRecord};
use composable_traits::{
	vault::{VaultConfig},
};
use std::collections::BTreeMap;
use sp_runtime::Perquintill;
use frame_support::{
	traits::{
		fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
		fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
	},
};


fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

const A: u128 = 2;
const B: u128 = 2000;

// if to make it generic, and pass u128, it will pass HasCompact, and u128 will be 5 bits, not 16...
pub fn recode_unwrap_u128<
	O: Decode + MaxEncodedLen + Encode,
	I: Decode + MaxEncodedLen + Encode,
>(
	raw: I,
) -> O {
	// next does not holds, because in wasm it is 16 and 8, in native 16 and 5. But that works fine
	// overall assert_eq!(I::max_encoded_len(), O::max_encoded_len(), "<I as
	// MaxEncodedLen>::max_encoded_len() must be equal <O as MaxEncodedLen>::max_encoded_len()");
	O::decode(&mut &raw.encode()[..]).unwrap()
}


benchmarks! {
	create_asset_vault_benchmark {
		let caller: T::AccountId = whitelisted_caller();
		T::Assets::mint_into(recode_unwrap_u128(A), &caller, 2u32.into())?;
		T::Assets::mint_into(recode_unwrap_u128(B), &caller, 1u32.into())?;

		let vault_config: VaultConfig<T::AccountId, T::MayBeAssetId> = VaultConfig {
			asset_id: recode_unwrap_u128(B),
			manager: whitelisted_caller(),
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		};
	}: { 
		TokenizedOptions::<T>::create_asset_vault(
			Origin::Signed(caller.clone()), 
			vault_config,
		)
	}

	create_asset_vault {
		let caller: T::AccountId = whitelisted_caller();
		T::Assets::mint_into(recode_unwrap_u128(A), &caller, 2u32.into())?;
		T::Assets::mint_into(recode_unwrap_u128(B), &caller, 1u32.into())?;

		let vault_config: VaultConfig<T::AccountId, T::MayBeAssetId> = VaultConfig {
			asset_id: recode_unwrap_u128(B),
			manager: whitelisted_caller(),
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		};
	}: _(
			RawOrigin::Signed(caller.clone()), 
			vault_config,
		)
}



impl_benchmark_test_suite!(
	TokenizedOptions,
	crate::mock::runtime::ExtBuilder::default().build(),
	crate::mock::runtime::MockRuntime,
);
  