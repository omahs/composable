use crate::mock::runtime::{
	accounts::*, AssetId, Balance, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions,
};
use crate::tests::*;
use crate::{pallet, Error, OptionIdToOption};
use frame_support::{assert_err, assert_noop, assert_ok};

use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use frame_system::ensure_signed;

// ----------------------------------------------------------------------------------------------------
//		Create Options Tests
// ----------------------------------------------------------------------------------------------------
/// Create BTC vault, create BTC option and check if option_id is correctly saved and event emitted
#[test]
fn test_create_option_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		// Get BTC and USDC vault config
		let btc_vault_config = VaultConfigBuilder::default().build();
		let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC::ID).build();

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
		let option_id = trait_create_option(Origin::signed(ADMIN), option_config.clone())
			.expect("Error creating option");

		// Check option has been created
		assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id,
			option_config,
		}));
	});
}

/// Create BTC vault, create BTC option and check if vault_id is correctly saved and event emitted using exstrinsic
#[test]
fn test_create_option_and_emit_event_ext() {
	ExtBuilder::default().build().execute_with(|| {
		// Get BTC and USDC vault config
		let btc_vault_config = VaultConfigBuilder::default().build();
		let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC::ID).build();

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
fn test_create_option_without_vaults_and_raise_error_ext() {
	ExtBuilder::default().build().execute_with(|| {
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
fn test_create_same_option_and_raise_error() {
	ExtBuilder::default().build().execute_with(|| {
		// Get BTC and USDC vault config
		let btc_vault_config = VaultConfigBuilder::default().build();
		let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC::ID).build();

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
fn test_create_same_option_and_raise_error_ext() {
	ExtBuilder::default().build().execute_with(|| {
		// Get BTC and USDC vault config
		let btc_vault_config = VaultConfigBuilder::default().build();
		let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC::ID).build();

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

proptest! {
	#![proptest_config(ProptestConfig::with_cases(10))]

	#[test]
	fn proptest_create_option(random_option_configs in prop_random_option_config_vec()) {
		ExtBuilder::default().build().initialize_all_vaults().execute_with(|| {
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
