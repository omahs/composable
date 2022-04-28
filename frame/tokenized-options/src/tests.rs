use crate::mock::currency::defs::*;
use crate::mock::runtime::{
	accounts::*, AssetId, Balance, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions,
};
use crate::OptionIdToOption;
use crate::{pallet, pallet::AssetToVault, pallet::Error, ExerciseType, OptionToken, OptionType};
use composable_traits::{tokenized_options::TokenizedOptions as OptionsTrait, vault::VaultConfig};
use frame_support::{assert_noop, assert_ok};
use frame_system::ensure_signed;
use itertools::Itertools;
use proptest::{
	prelude::*,
	prop_oneof,
	strategy::{Just, Strategy},
};
use sp_runtime::Perquintill;
use std::collections::BTreeMap;

// ----------------------------------------------------------------------------------------------------
//		Setup VaultConfigBuilder
// ----------------------------------------------------------------------------------------------------
struct VaultConfigBuilder {
	pub asset_id: AssetId,
	pub manager: AccountId,
	// pub reserved: Perquintill,
	// pub strategies: BTreeMap<AccountId, Perquintill>,
}

impl Default for VaultConfigBuilder {
	fn default() -> Self {
		VaultConfigBuilder {
			asset_id: BTC::ID,
			manager: ADMIN,
			// reserved: Perquintill::one(),
			// strategies: BTreeMap::new(),
		}
	}
}

impl VaultConfigBuilder {
	fn build(self) -> VaultConfig<AccountId, AssetId> {
		VaultConfig {
			asset_id: self.asset_id,
			manager: self.manager,
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		}
	}

	fn asset_id(mut self, asset: AssetId) -> Self {
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
	fn build(self) -> OptionToken<AssetId, Balance, Moment> {
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
//		Extrinsic Simulations
// ----------------------------------------------------------------------------------------------------

// Simulate exstrinsic call `create_option`, but returning values
fn trait_create_option(
	_origin: Origin,
	_option: &OptionToken<AssetId, Balance, Moment>,
) -> AssetId {
	let account_id = ensure_signed(_origin).unwrap();

	let option_id = <TokenizedOptions as OptionsTrait>::create_option(account_id, _option).unwrap();

	TokenizedOptions::deposit_event(pallet::Event::CreatedOption {
		option_id,
		option: _option.clone(),
	});

	option_id
}

// Simulate exstrinsic call `sell_option`, but returning values
fn trait_sell_option(_origin: Origin, _amount: Balance, _option_id: AssetId) -> () {
	let account_id = ensure_signed(_origin).unwrap();

	// Not yet correctly implemented
	<TokenizedOptions as OptionsTrait>::sell_option(&account_id, _amount, _option_id).unwrap();

	TokenizedOptions::deposit_event(pallet::Event::SellOption {
		who: account_id,
		amount: _amount,
		option_id: _option_id,
	});

	()
}

// ----------------------------------------------------------------------------------------------------
//		Prop Compose
// ----------------------------------------------------------------------------------------------------
#[allow(dead_code)]
pub fn pick_currency() -> impl Strategy<Value = AssetId> {
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
	) -> Vec<AssetId>{
		assets
   }
}

prop_compose! {
	fn generate_balances()(
		balances in prop::collection::vec(MINIMUM_RESERVE..MAXIMUM_RESERVE, 1..=TOTAL_NUM_OF_ASSETS),
	) -> Vec<Balance>{
		balances
   }
}

prop_compose! {
	fn generate_prices()(
		prices in prop::collection::vec(MINIMUM_STRIKE_PRICE..MAXIMUM_STRIKE_PRICE, 1..=TOTAL_NUM_OF_ASSETS),
	) -> Vec<Balance>{
		prices
   }
}

prop_compose! {
	fn generate_markets()(assets in generate_assets(), prices in generate_prices()) -> Vec<(AssetId, Balance)>{
		assets.into_iter().unique().zip(prices.into_iter()).collect()
	}
}

prop_compose! {
	fn generate_blockchain_state()(
		accounts in generate_accounts(),
		assets in generate_assets(),
		balances in generate_balances(),
		asset_prices in generate_prices()
	) -> Vec<(AccountId, AssetId, Balance, Balance)>{
		accounts.into_iter()
			.zip(assets.into_iter())
			.unique()
			.zip(balances.into_iter())
			.unique()
			.zip(asset_prices.into_iter())
			.map(|(((account, asset), balance), asset_price)| (account, asset, balance, asset_price))
			.collect()
   }
}

// ----------------------------------------------------------------------------------------------------
//		Create Vault Tests
// ----------------------------------------------------------------------------------------------------
#[test]
fn test_create_vault_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		// Get default vault config
		let vault_config = VaultConfigBuilder::default().build();

		// Create vault
		assert_ok!(TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), vault_config));

		// Check vault has been created
		assert!(AssetToVault::<MockRuntime>::contains_key(BTC::ID));

		// Get created vault_id
		let vault_id = TokenizedOptions::asset_id_to_vault_id(BTC::ID).unwrap();

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id,
			asset_id: BTC::ID,
		}));
	});
}

#[test]
fn test_create_same_vault_and_emit_error() {
	ExtBuilder::default().build().execute_with(|| {
		// Get default vault config
		let vault_config = VaultConfigBuilder::default().build();

		// Create vault
		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			vault_config.clone()
		));

		// Check vault has been created
		assert!(AssetToVault::<MockRuntime>::contains_key(BTC::ID));

		// Get created vault_id
		let vault_id = TokenizedOptions::asset_id_to_vault_id(BTC::ID).unwrap();

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id,
			asset_id: BTC::ID,
		}));

		// Create same vault again and check error is raised
		assert_noop!(
			TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), vault_config.clone()),
			Error::<MockRuntime>::AssetVaultAlreadyExists
		);
	});
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(100))]
	#[test]
	fn proptest_create_vault(assets in generate_assets()) {
		ExtBuilder::default().build().execute_with(|| {
			assets.iter().for_each(|&asset| {
				// Get vault config with custom asset
				let config = VaultConfigBuilder::default().asset_id(asset).build();

				// Check if vault has not been already created and create it or raise error
				if !AssetToVault::<MockRuntime>::contains_key(config.asset_id) {
					assert_ok!(TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), config.clone()));
					assert!(AssetToVault::<MockRuntime>::contains_key(config.asset_id));
					let vault_id = TokenizedOptions::asset_id_to_vault_id(config.asset_id).unwrap();
					System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
						vault_id,
						asset_id: config.asset_id,
					}));
				} else {
					assert_noop!(
						TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), config.clone()),
						Error::<MockRuntime>::AssetVaultAlreadyExists
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
		// Get default option
		let option = OptionsBuilder::default().build();

		// Create option and get option id
		let option_id = trait_create_option(Origin::signed(ADMIN), &option);

		// Check option has been created
		assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id,
			option,
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
				let option_id = trait_create_option(Origin::signed(ADMIN), &option);

				assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));

				System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
					option_id,
					option: option.clone(),
				}));
			})
		});
	}
}

// ----------------------------------------------------------------------------------------------------
//		Sell/Buy Options Tests
// ----------------------------------------------------------------------------------------------------
#[test]
fn test_sell_option_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		let option_btc = OptionsBuilder::default().build();

		assert_ok!(TokenizedOptions::create_option(Origin::signed(ADMIN), option_btc.clone()));

		assert!(OptionIdToOption::<MockRuntime>::contains_key(100_000_000_001u128));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id: 100_000_000_001u128,
			option: option_btc.clone(),
		}));

		assert_ok!(TokenizedOptions::sell_option(
			Origin::signed(ADMIN),
			1u128,
			100_000_000_001u128
		));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::SellOption {
			option_id: 100_000_000_001u128,
			who: ADMIN,
			amount: 1u128,
		}));
	});
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(100))]
	#[test]
	fn proptest_sell_option(blockchain_state in generate_blockchain_state()) {

		let balances: Vec<(AccountId, AssetId, Balance)> = blockchain_state.into_iter().map(|(account, asset, balance, price)| (account, asset, balance)).collect();

		ExtBuilder::default().init_balances(balances.clone()).build().execute_with(|| {

			// let options: Vec<OptionToken<AssetId, Balance>> = balances.iter().map(|&(_, asset, price)| {
			// 	OptionsBuilder::default().base_asset_id(asset).strike_price(price).build()
			// }).collect();

			// options.iter().for_each(|option|{
			// 	assert_ok!(TokenizedOptions::create_option_with_vault(Origin::signed(ADMIN), option.clone()));
			// 	let (option_id, vault_id) =
			// 	trait_create_option_with_vault(Origin::signed(ADMIN), &option);

			// 	assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));
			// 	assert!(OptionToVault::<MockRuntime>::contains_key(option_id));

			// 	System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOptionVault {
			// 		option_id,
			// 		vault_id,
			// 		option: option.clone(),
			// 	}));
			// })
		});
	}
}
