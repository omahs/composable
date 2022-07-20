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

fn valid_option_config<T: Config>() -> OptionConfigOf<T> {
	OptionConfigOf::<T> {
		base_asset_id: recode_unwrap_u128(B),
		quote_asset_id: recode_unwrap_u128(C),
		base_asset_strike_price: BalanceOf::<T>::from(50000u128 * UNIT),
		quote_asset_strike_price: UNIT.into(),
		option_type: OptionType::Call,
		exercise_type: ExerciseType::European,
		expiring_date: recode_unwrap_u128(6000u64),
		// Use this when https://github.com/paritytech/substrate/pull/10128 is merged
		// epoch: Epoch { 
		// 	deposit: recode_unwrap_u128(0u64), 
		// 	purchase: recode_unwrap_u128(3000u64), 
		// 	exercise: recode_unwrap_u128(6000u64), 
		// 	end: recode_unwrap_u128(9000u64) 
		// },
		epoch: Epoch { 
			deposit: recode_unwrap_u128(0u64), 
			purchase: recode_unwrap_u128(2000u64), 
			exercise: recode_unwrap_u128(5000u64), 
			end: recode_unwrap_u128(9000u64) 
		},
		base_asset_amount_per_option: UNIT.into(),
		quote_asset_amount_per_option: UNIT.into(),
		total_issuance_seller: 0u128.into(),
		total_premium_paid: 0u128.into(),
		exercise_amount: 0u128.into(),
		base_asset_spot_price: 0u128.into(),
		total_issuance_buyer: 0u128.into(),
		total_shares_amount: 0u128.into(),
	}
}


fn default_option_benchmarking_setup<T: Config>() -> OptionIdOf<T> {
	let origin = OriginFor::<T>::from(RawOrigin::Root);

	let option_config: OptionConfigOf<T> = valid_option_config::<T>();

	TokenizedOptions::<T>::create_option(
		origin, 
		option_config.clone(),
	).unwrap();

	let option_hash = TokenizedOptions::<T>::generate_id(
		option_config.base_asset_id,
		option_config.quote_asset_id,
		option_config.base_asset_strike_price,
		option_config.quote_asset_strike_price,
		option_config.option_type,
		option_config.expiring_date,
		option_config.exercise_type,
	);

	OptionHashToOptionId::<T>::get(option_hash).unwrap()
}

// ----------------------------------------------------------------------------------------------------
//		Benchmark tests
// ----------------------------------------------------------------------------------------------------

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

	create_option {
		let origin = OriginFor::<T>::from(RawOrigin::Root);

		vault_benchmarking_setup::<T>(recode_unwrap_u128(B), 50_000);
		vault_benchmarking_setup::<T>(recode_unwrap_u128(C), 1);

		let option_config = OptionConfigOf::<T> {
			base_asset_id: recode_unwrap_u128(B),
			quote_asset_id: recode_unwrap_u128(C),
			base_asset_strike_price: BalanceOf::<T>::from(50000u128 * UNIT),
			quote_asset_strike_price: UNIT.into(),
			option_type: OptionType::Call,
			exercise_type: ExerciseType::European,
			expiring_date: recode_unwrap_u128(6000u64),
			epoch: Epoch { 
				deposit: recode_unwrap_u128(0u64), 
				purchase: recode_unwrap_u128(3000u64), 
				exercise: recode_unwrap_u128(6000u64), 
				end: recode_unwrap_u128(9000u64) 
			},
			base_asset_amount_per_option: UNIT.into(),
			quote_asset_amount_per_option: UNIT.into(),
			total_issuance_seller: 0u128.into(),
			total_premium_paid: 0u128.into(),
			exercise_amount: 0u128.into(),
			base_asset_spot_price: 0u128.into(),
			total_issuance_buyer: 0u128.into(),
			total_shares_amount: 0u128.into(),
		};
	}: {
		TokenizedOptions::<T>::create_option(
			origin, 
			option_config.clone(),
		)?
	}
	verify {
		assert_last_event::<T>(Event::CreatedOption {
			// First 1..01 and 1..02 are for vaults lp_tokens
			option_id: recode_unwrap_u128(100000000003u128),
            option_config,
		}.into())
	}

}

impl_benchmark_test_suite!(
	TokenizedOptions,
	crate::mock::runtime::ExtBuilder::default().build(),
	crate::mock::runtime::MockRuntime,
);
