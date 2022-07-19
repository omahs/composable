//! Benchmarks for Template Pallet
// #![cfg(feature = "runtime-benchmarks")]

use crate::*;
use crate::{Pallet as TokenizedOptions, self as pallet_tokenized_options, types::*};

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::{EventRecord, Pallet as System, RawOrigin};
use composable_traits::{
	vault::{VaultConfig},
	defi::{DeFiComposableConfig},
	oracle::Price
};
use std::collections::BTreeMap;
use sp_runtime::Perquintill;
use frame_support::{
	traits::{EnsureOrigin,
		fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
		fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
	},
};
use frame_system::pallet_prelude::*;



// ----------------------------------------------------------------------------------------------------
//		Helper functions
// ----------------------------------------------------------------------------------------------------
const UNIT: u128 = 10u128.pow(12);
const A: u128 = 2;
const B: u128 = 2000;
const C: u128 = 131;

fn encode_decode<D: Decode, E: Encode>(value: E) -> D {
	let asset_id = value.encode();
	D::decode(&mut &asset_id[..]).unwrap()
}

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

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn set_oracle_price<T: Config + pallet_oracle::Config>(asset_id: T::MayBeAssetId, price: u64) {
	let asset_id: T::AssetId = encode_decode(asset_id);

	pallet_oracle::Prices::<T>::insert(
		asset_id,
		Price { price: <T as pallet_oracle::Config>::PriceValue::from(price), block: 0_u32.into() },
	);
}

fn vault_benchmarking_setup<T: Config + pallet_oracle::Config>(asset_id: T::MayBeAssetId, price: u64)  {
	let origin = OriginFor::<T>::from(RawOrigin::Root);

	set_oracle_price::<T>(asset_id, price * (UNIT as u64));


	let vault_config: VaultConfig<T::AccountId, T::MayBeAssetId> = VaultConfig {
		asset_id,
		manager: whitelisted_caller(),
		reserved: Perquintill::one(),
		strategies: BTreeMap::new(),
	};

	TokenizedOptions::<T>::create_asset_vault(
		origin, 
		vault_config,
	).unwrap();
}


benchmarks! {
	where_clause {
		where
			T: pallet_tokenized_options::Config
				+ DeFiComposableConfig 
				+ frame_system::Config
				+ pallet_oracle::Config
	}
	
	create_asset_vault {
		let origin = OriginFor::<T>::from(RawOrigin::Root);

		set_oracle_price::<T>(recode_unwrap_u128(B), 50_000 * (UNIT as u64));
		set_oracle_price::<T>(recode_unwrap_u128(C), UNIT as u64);

		let vault_config: VaultConfig<T::AccountId, T::MayBeAssetId> = VaultConfig {
			asset_id: recode_unwrap_u128(B),
			manager: whitelisted_caller(),
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		};
	}: {
		TokenizedOptions::<T>::create_asset_vault(
			origin, 
			vault_config,
		)?
	}
	verify {
		assert_last_event::<T>(Event::CreatedAssetVault {
			vault_id: recode_unwrap_u128(1u128),
            asset_id: recode_unwrap_u128(B),
		}.into())
	}


}

impl_benchmark_test_suite!(
	TokenizedOptions,
	crate::mock::runtime::ExtBuilder::default().build(),
	crate::mock::runtime::MockRuntime,
);
