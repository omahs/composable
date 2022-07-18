//! Benchmarks for Template Pallet
// #![cfg(feature = "runtime-benchmarks")]

use crate::*;
use crate::{Pallet as TokenizedOptions, self as pallet_tokenized_options};

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
use frame_system::{pallet_prelude::*};

fn encode_decode<D: Decode, E: Encode>(value: E) -> D {
	let asset_id = value.encode();
	let asset_id = D::decode(&mut &asset_id[..]).unwrap();
	asset_id
}


// fn set_oracle_price<T: pallet_oracle::Config>(asset_id: AssetIdOf<T>, balance: BalanceOf<T>) {
// 	let price = Price { price: balance, block: System::block_number() };
// 	pallet_oracle::Prices::<T>::insert(asset_id, price);
// }


fn set_oracle_price<T: Config + pallet_oracle::Config>(asset_id: T::MayBeAssetId, price: u64) {
	let asset_id: T::AssetId = encode_decode(asset_id);

	pallet_oracle::Prices::<T>::insert(
		asset_id,
		Price { price: <T as pallet_oracle::Config>::PriceValue::from(price), block: 0_u32.into() },
	);
}



fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

const UNIT: u64 = 10u64.pow(12);
const A: u128 = 2;
const B: u128 = 2000;
const C: u128 = 131;


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
	where_clause {
		where
			T: pallet_tokenized_options::Config
				+ DeFiComposableConfig 
				+ frame_system::Config
				+ pallet_oracle::Config
	}
	create_asset_vault {
		let caller: <T as frame_system::Config>::AccountId = whitelisted_caller::<T::AccountId>();
		// let origin = OriginFor::<T>::from(RawOrigin::Signed(caller));
		let origin = OriginFor::<T>::from(RawOrigin::Root);

		set_oracle_price::<T>(recode_unwrap_u128(B), 50_000 * UNIT);
		set_oracle_price::<T>(recode_unwrap_u128(C), UNIT);

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
