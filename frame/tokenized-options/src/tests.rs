use crate::currency::{defs::*, CurrencyId};
use crate::mock::{
	accounts::*, AssetId, Balance, Event, ExtBuilder, MockRuntime, Origin, System, TokenizedOptions,
};
use crate::{pallet, pallet::AssetToVault, pallet::Error};
use crate::{OptionIdToOption, OptionToVault};
use composable_traits::{
	tokenized_options::{ExerciseType, OptionToken, OptionType},
	vault::VaultConfig,
};
use frame_support::{assert_noop, assert_ok};
use itertools::Itertools;
use proptest::{
	prelude::*,
	prop_oneof,
	strategy::{Just, Strategy},
};
use sp_runtime::Perquintill;
use std::collections::BTreeMap;

// ----------------------------------------------------------------------------------------------------
//		Setup VaultBuilder
// ----------------------------------------------------------------------------------------------------
struct VaultConfigBuilder {
	pub asset_id: CurrencyId,
	pub manager: AccountId,
	pub reserved: Perquintill,
	pub strategies: BTreeMap<AccountId, Perquintill>,
}

impl Default for VaultConfigBuilder {
	fn default() -> Self {
		VaultConfigBuilder {
			asset_id: BTC::ID,
			manager: ADMIN,
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		}
	}
}

impl VaultConfigBuilder {
	fn build(self) -> VaultConfig<AccountId, CurrencyId> {
		VaultConfig {
			asset_id: self.asset_id,
			manager: self.manager,
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		}
	}

	fn asset_id(mut self, asset: CurrencyId) -> Self {
		self.asset_id = asset;
		self
	}

	fn manager(mut self, manager: AccountId) -> Self {
		self.manager = manager;
		self
	}
}

// ----------------------------------------------------------------------------------------------------
//		Setup OptionsBuilder
// ----------------------------------------------------------------------------------------------------
struct OptionsBuilder {
	pub base_asset_id: AssetId,
	pub strike_price: Balance,
	pub option_type: OptionType,
	pub exercise_type: ExerciseType,
}

impl Default for OptionsBuilder {
	fn default() -> Self {
		OptionsBuilder {
			base_asset_id: BTC::ID,
			strike_price: 50000u128,
			option_type: OptionType::Call,
			exercise_type: ExerciseType::European,
		}
	}
}

impl OptionsBuilder {
	fn build(self) -> OptionToken<AssetId, Balance> {
		OptionToken {
			base_asset_id: self.base_asset_id,
			strike_price: self.strike_price,
			option_type: OptionType::Call,
			exercise_type: ExerciseType::European,
		}
	}

	fn base_asset_id(mut self, base_asset_id: AssetId) -> Self {
		self.base_asset_id = base_asset_id;
		self
	}

	fn strike_price(mut self, strike_price: Balance) -> Self {
		self.strike_price = strike_price;
		self
	}
}

// ----------------------------------------------------------------------------------------------------
//		Pick Functions
// ----------------------------------------------------------------------------------------------------
#[allow(dead_code)]
pub fn pick_currency() -> impl Strategy<Value = CurrencyId> {
	prop_oneof![
		Just(PICA::ID),
		Just(USDC::ID),
		Just(BTC::ID),
		Just(LAYR::ID),
		Just(DOT::ID),
		Just(KSM::ID),
		Just(ETH::ID),
	]
}

#[allow(dead_code)]
pub fn pick_account() -> impl Strategy<Value = AccountId> {
	prop_oneof![Just(ALICE), Just(BOB), Just(CHARLIE), Just(DAVE), Just(EVEN),]
}
// ----------------------------------------------------------------------------------------------------
//		Prop Compose
// ----------------------------------------------------------------------------------------------------
#[allow(dead_code)]
const MINIMUM_RESERVE: Balance = 1_000;

#[allow(dead_code)]
const MAXIMUM_RESERVE: Balance = 1_000_000_000;

#[allow(dead_code)]
const MINIMUM_STRIKE_PRICE: Balance = 1;

#[allow(dead_code)]
const MAXIMUM_STRIKE_PRICE: Balance = 1_000_000;

#[allow(dead_code)]
const TOTAL_NUM_OF_ASSETS: usize = 7;

#[allow(dead_code)]
const TOTAL_NUM_OF_ACCOUNTS: usize = 5;

#[allow(dead_code)]
const NUMBER_OF_PROPTEST_CASES: u32 =
	3u32 * TOTAL_NUM_OF_ASSETS as u32 * TOTAL_NUM_OF_ACCOUNTS as u32;

prop_compose! {
	fn generate_accounts()(
		accounts in prop::collection::vec(pick_account(), 1..=TOTAL_NUM_OF_ACCOUNTS),
	) -> Vec<AccountId>{
		accounts
   }
}

prop_compose! {
	fn generate_assets()(
		assets in prop::collection::vec(pick_currency(), 1..=TOTAL_NUM_OF_ASSETS),
	) -> Vec<CurrencyId>{
		assets
   }
}

prop_compose! {
	fn generate_strike_prices()(
		strike_prices in prop::collection::vec(MINIMUM_STRIKE_PRICE..MAXIMUM_STRIKE_PRICE, 1..=TOTAL_NUM_OF_ASSETS),
	) -> Vec<Balance>{
		strike_prices
   }
}

prop_compose! {
	fn generate_markets()(assets in generate_assets(), strike_prices in generate_strike_prices()) -> Vec<(AssetId, Balance)>{
		assets.into_iter().unique().zip(strike_prices.into_iter()).collect()
	}
}

// ----------------------------------------------------------------------------------------------------
//		Create Vault Tests
// ----------------------------------------------------------------------------------------------------
#[test]
fn test_create_vault_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		let btc_vault = VaultConfigBuilder::default().build();

		assert_ok!(TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), btc_vault.clone()));

		assert!(AssetToVault::<MockRuntime>::contains_key(BTC::ID));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id: 1u64,
			asset_id: BTC::ID,
		}));
	});
}

#[test]
fn test_create_same_vault_and_emit_error() {
	ExtBuilder::default().build().execute_with(|| {
		let btc_vault = VaultConfigBuilder::default().build();

		assert_ok!(TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), btc_vault.clone()));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id: 1u64,
			asset_id: BTC::ID,
		}));

		assert_noop!(
			TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), btc_vault.clone()),
			Error::<MockRuntime>::VaultAlreadyExists
		);
	});
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(100))]
	#[test]
	fn proptest_create_vault_and_emit_event_or_error(assets in generate_assets()) {
		ExtBuilder::default().build().execute_with(|| {
			assets.iter().for_each(|&asset| {

				let config = VaultConfigBuilder::default().asset_id(asset).build();

				if !AssetToVault::<MockRuntime>::contains_key(config.asset_id) {
					assert_ok!(TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), config.clone()));
					assert!(AssetToVault::<MockRuntime>::contains_key(config.asset_id));
				} else {
					assert_noop!(
						TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), config.clone()),
						Error::<MockRuntime>::VaultAlreadyExists
					);
				}
			});
		});
	}
}

// ----------------------------------------------------------------------------------------------------
//		Create Options Tests
// ----------------------------------------------------------------------------------------------------
#[test]
fn test_create_option_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		let option_btc = OptionsBuilder::default().build();

		assert_ok!(TokenizedOptions::create_option(Origin::signed(ADMIN), option_btc.clone()));

		assert!(OptionIdToOption::<MockRuntime>::contains_key(100_000_000_001u128));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id: 100_000_000_001u128,
			option: option_btc,
		}));
	});
}

#[test]
fn test_create_option_with_vault_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		let option_btc = OptionsBuilder::default().build();

		assert_ok!(TokenizedOptions::create_option_with_vault(
			Origin::signed(ADMIN),
			option_btc.clone()
		));

		assert!(OptionIdToOption::<MockRuntime>::contains_key(100_000_000_001u128));
		assert!(OptionToVault::<MockRuntime>::contains_key(100_000_000_001u128));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOptionVault {
			option_id: 100_000_000_001u128,
			vault_id: 1u64,
			option: option_btc.clone(),
		}));
	});
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(100))]
	#[test]
	fn proptest_create_option(market_prices in generate_markets()) {
		ExtBuilder::default().build().execute_with(|| {
			let options: Vec<OptionToken<AssetId, Balance>> = market_prices.iter().map(|&(asset, strike_price)| {
				OptionsBuilder::default().base_asset_id(asset).strike_price(strike_price).build()
			}).collect();

			options.iter().for_each(|option|{
				assert_ok!(TokenizedOptions::create_option(Origin::signed(ADMIN), option.clone()));

				// How to get the created value and make checks?
			})
		});
	}
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(100))]
	#[test]
	fn proptest_create_option_with_vault(market_prices in generate_markets()) {
		ExtBuilder::default().build().execute_with(|| {
			let options: Vec<OptionToken<AssetId, Balance>> = market_prices.iter().map(|&(asset, strike_price)| {
				OptionsBuilder::default().base_asset_id(asset).strike_price(strike_price).build()
			}).collect();

			options.iter().for_each(|option|{
				assert_ok!(TokenizedOptions::create_option_with_vault(Origin::signed(ADMIN), option.clone()));

				// How to get the created value and make checks?
			})
		});
	}
}
