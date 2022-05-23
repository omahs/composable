use crate::mock::accounts::*;
use crate::mock::assets::*;
use crate::mock::runtime::{
	Balance, Event, ExtBuilder, MockRuntime, Moment, Origin, System, TokenizedOptions,
};
use crate::tests::*;
use crate::{pallet, Error, OptionHashToOptionId, OptionIdToOption};
use frame_support::{assert_err, assert_noop, assert_ok};

use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use frame_system::ensure_signed;

// ----------------------------------------------------------------------------------------------------
//		Create Options Tests
// ----------------------------------------------------------------------------------------------------
/// Create BTC vault, create BTC option and check if option_id is correctly saved and event emitted
#[test]
fn test_create_option_success() {
	ExtBuilder::default().build().initialize_oracle_prices().execute_with(|| {
		// Get BTC and USDC vault config
		let btc_vault_config = VaultConfigBuilder::default().build();
		let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC).build();

		// Create BTC and USDC vaults
		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			btc_vault_config.clone()
		));

		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			usdc_vault_config.clone(),
		));

		// Get BTC option config
		let option_config = OptionsConfigBuilder::default().build();

		let option_hash = TokenizedOptions::generate_id(
			option_config.base_asset_id,
			option_config.base_asset_strike_price,
			option_config.option_type,
			option_config.expiring_date,
		);

		// Create option and get option id
		let option_id = trait_create_option(Origin::signed(ADMIN), option_config.clone())
			.expect("Error creating option");

		// Check option has been created
		assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));
		assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));
		let option_id_from_hash = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();
		assert_eq!(option_id, option_id_from_hash);

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id,
			option_config,
		}));
	});
}

/// Create BTC vault, create BTC option and check if vault_id is correctly saved and event emitted using exstrinsic
#[test]
fn test_create_option_success_ext() {
	ExtBuilder::default().build().initialize_oracle_prices().execute_with(|| {
		// Get BTC and USDC vault config
		let btc_vault_config = VaultConfigBuilder::default().build();
		let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC).build();

		// Create BTC and USDC vaults
		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			btc_vault_config.clone()
		));

		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			usdc_vault_config.clone(),
		));

		// Get BTC option config
		let option_config = OptionsConfigBuilder::default().build();

		// Create option and get option id
		assert_ok!(TokenizedOptions::create_option(Origin::signed(ADMIN), option_config.clone()));

		// Check option has been created (ID = 3 because first two IDs are used for the vaults lp_tokens)
		assert!(OptionIdToOption::<MockRuntime>::contains_key(100000000003u128));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id: 100000000003u128,
			option_config,
		}));
	});
}

#[test]
fn test_create_option_error_vaults_not_exist_ext() {
	ExtBuilder::default().build().initialize_oracle_prices().execute_with(|| {
		// Get default option config
		let option_config = OptionsConfigBuilder::default().build();

		// Create same option again and check error is raised
		assert_noop!(
			TokenizedOptions::create_option(Origin::signed(ADMIN), option_config.clone()),
			Error::<MockRuntime>::OptionAssetVaultsDoNotExist
		);

		// Check option has not been created
		assert!(!OptionIdToOption::<MockRuntime>::contains_key(100000000001u128));
	});
}

/// Create BTC vault, create BTC option twice and check if error is correctly raised and storage not changed
#[test]
fn test_create_option_error_option_already_exists() {
	ExtBuilder::default().build().initialize_oracle_prices().execute_with(|| {
		// Get BTC and USDC vault config
		let btc_vault_config = VaultConfigBuilder::default().build();
		let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC).build();

		// Create BTC and USDC vaults
		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			btc_vault_config.clone()
		));

		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			usdc_vault_config.clone(),
		));

		// Get default option config
		let option_config = OptionsConfigBuilder::default().build();

		let option_id = trait_create_option(Origin::signed(ADMIN), option_config.clone())
			.expect("Error creating option");

		// Check option has been created
		assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id,
			option_config: option_config.clone(),
		}));

		// Create same option again and check error is raised
		assert_noop!(
			TokenizedOptions::create_option(Origin::signed(ADMIN), option_config.clone()),
			Error::<MockRuntime>::OptionIdAlreadyExists
		);
	});
}

/// Create BTC vault, create BTC option twice and check if error is correctly raised and storage not changed using extrinsic
#[test]
fn test_create_option_error_option_already_exists_ext() {
	ExtBuilder::default().build().initialize_oracle_prices().execute_with(|| {
		// Get BTC and USDC vault config
		let btc_vault_config = VaultConfigBuilder::default().build();
		let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC).build();

		// Create BTC and USDC vaults
		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			btc_vault_config.clone()
		));

		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			usdc_vault_config.clone(),
		));

		// Get default option config
		let option_config = OptionsConfigBuilder::default().build();

		assert_ok!(TokenizedOptions::create_option(Origin::signed(ADMIN), option_config.clone()));

		// Check option has been created
		assert!(OptionIdToOption::<MockRuntime>::contains_key(100000000003u128));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id: 100000000003u128,
			option_config: option_config.clone(),
		}));

		// Create same option again and check error is raised
		assert_noop!(
			TokenizedOptions::create_option(Origin::signed(ADMIN), option_config.clone()),
			Error::<MockRuntime>::OptionIdAlreadyExists
		);
	});
}

// TODO: create option with no-admin account and check error raised

// TODO: implement validation of various option attributes and make tests
//		- base_asset != quote_asset and are both supported by oracle
//		- timestamps (expiry date, epoch windows, etc) are not in the past (or other useful checks)
// 		- total issuances for sale and buy are 0 at the start

proptest! {
	#![proptest_config(ProptestConfig::with_cases(20))]
	#[test]
	fn proptest_create_option(random_option_configs in prop_random_option_config_vec()) {
		// Create all the asset vaults before creating options
		ExtBuilder::default().build().initialize_oracle_prices().initialize_all_vaults().execute_with(|| {
			random_option_configs.iter().for_each(|option_config|{

				let option_config = OptionsConfigBuilder::default().base_asset_id(option_config.0).base_asset_strike_price(option_config.1).build();

				match trait_create_option(Origin::signed(ADMIN), option_config.clone()) {
					Ok(option_id) => {
						assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));

						System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
							option_id,
							option_config,
						}));
					},
					Err(error) => {
						assert_eq!(error, DispatchError::from(Error::<MockRuntime>::OptionAssetVaultsDoNotExist));
					}
				};
			})
		});
	}
}
