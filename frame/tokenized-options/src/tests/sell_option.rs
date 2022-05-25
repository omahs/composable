use crate::mock::runtime::{
	Assets, Balance, Balances, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions, Vault,
};

use crate::mock::accounts::*;
use crate::mock::assets::*;

use crate::pallet::{self, OptionHashToOptionId, Sellers};
use crate::tests::*;

use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use composable_traits::vault::Vault as VaultTrait;

use frame_support::assert_noop;
use frame_support::traits::fungibles::Inspect;
use frame_system::ensure_signed;
use sp_core::{sr25519::Public, H256};

// ----------------------------------------------------------------------------------------------------
//		Sell Options Tests
// ----------------------------------------------------------------------------------------------------

fn sell_option_success_checks(
	option_hash: H256,
	option_config: OptionConfig<AssetId, Balance, Moment>,
	option_amount: Balance,
	who: Public,
) {
	// Get info before extrinsic for checks
	let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();
	let vault_id = AssetToVault::<MockRuntime>::get(option_config.base_asset_id).unwrap();
	let lp_token_id = <Vault as VaultTrait>::lp_asset_id(&vault_id).unwrap();
	let protocol_account = TokenizedOptions::account_id(option_config.base_asset_id);
	let asset_amount = option_amount * 10u128.pow(12);
	let shares_amount =
		<Vault as VaultTrait>::calculate_lp_tokens_to_mint(&vault_id, asset_amount).unwrap();

	let initial_user_balance = Assets::balance(option_config.base_asset_id, &who);
	let initial_vault_balance =
		Assets::balance(option_config.base_asset_id, &Vault::account_id(&vault_id));
	let initial_user_position =
		Sellers::<MockRuntime>::try_get(option_id, who).unwrap_or(SellerPosition::default());

	// Call exstrinsic
	assert_ok!(TokenizedOptions::sell_option(Origin::signed(who), option_amount, option_id));

	// Check correct event
	System::assert_last_event(Event::TokenizedOptions(pallet::Event::SellOption {
		seller: who,
		option_amount,
		option_id,
	}));

	// Check seller position is saved
	assert!(Sellers::<MockRuntime>::contains_key(option_id, who));

	// Check seller balance after sale is empty
	assert_eq!(
		Assets::balance(option_config.base_asset_id, &who),
		initial_user_balance - asset_amount
	);

	// Check vault balance after sale is correct
	assert_eq!(
		Assets::balance(option_config.base_asset_id, &Vault::account_id(&vault_id)),
		initial_vault_balance + asset_amount
	);

	// Check protocol owns all the issuance of lp_token
	assert_eq!(
		Assets::balance(lp_token_id, &protocol_account),
		Assets::total_issuance(lp_token_id)
	);

	// Check position is updated correctly
	let updated_user_position =
		Sellers::<MockRuntime>::try_get(option_id, who).unwrap_or(SellerPosition::default());

	assert_eq!(
		updated_user_position.option_amount,
		initial_user_position.option_amount + option_amount,
	);
	assert_eq!(
		updated_user_position.shares_amount,
		initial_user_position.shares_amount + shares_amount,
	);
}

#[test]
fn test_sell_option_with_initialization_success() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([(BOB, BTC, 1 * 10u128.pow(12))]))
		.build()
		.initialize_oracle_prices()
		.execute_with(|| {
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

			// Create default BTC option
			let option_config = OptionsConfigBuilder::default().build();

			assert_ok!(TokenizedOptions::create_option(
				Origin::signed(ADMIN),
				option_config.clone()
			));

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.base_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
			);

			// Check creation ended correctly
			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			// Perform extrinsic and make checks
			let option_amount = 1u128;
			sell_option_success_checks(option_hash, option_config, option_amount, BOB);
		});
}

#[test]
fn test_sell_option_success() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([(BOB, BTC, 7 * 10u128.pow(12))]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.base_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_amount = 7u128; // Same as BOB's balance

			sell_option_success_checks(option_hash, option_config, option_amount, BOB);
		});
}

#[test]
fn test_sell_option_update_position() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([(BOB, BTC, 5 * 10u128.pow(12))]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.base_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_amount = 3u128;

			sell_option_success_checks(option_hash, option_config.clone(), option_amount, BOB);

			let option_amount = 1u128;

			sell_option_success_checks(option_hash, option_config, option_amount, BOB);
		});
}

#[test]
fn test_sell_option_multiple_users() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 10 * 10u128.pow(12)),
			(BOB, BTC, 7 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.base_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let alice_option_amount = 7u128;
			sell_option_success_checks(
				option_hash,
				option_config.clone(),
				alice_option_amount,
				ALICE,
			);

			let bob_option_amount = 4u128;
			sell_option_success_checks(option_hash, option_config.clone(), bob_option_amount, BOB);

			let alice_option_amount = 2u128;
			sell_option_success_checks(
				option_hash,
				option_config.clone(),
				alice_option_amount,
				ALICE,
			);

			let bob_option_amount = 3u128;
			sell_option_success_checks(option_hash, option_config, bob_option_amount, BOB);
		});
}

#[test]
fn test_sell_option_error_option_not_exists() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([(BOB, BTC, 1 * 10u128.pow(12))]))
		.build()
		.execute_with(|| {
			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), 1u128, 10000000000005u128), // Not-existent option_id
				Error::<MockRuntime>::OptionIdDoesNotExists
			);
		});
}

#[test]
fn test_sell_option_error_user_has_not_enough_funds() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([(BOB, BTC, 5 * 10u128.pow(12))]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.base_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), 8u128, option_id),
				Error::<MockRuntime>::UserHasNotEnoughFundsToDeposit
			);
		});
}

// proptest! {
// 	#![proptest_config(ProptestConfig::with_cases(20))]
// 	#[test]
// 	fn proptest_sell_option(random_option_configs in prop_random_option_config_vec()) {
// 		// Create all the asset vaults before creating options
// 		ExtBuilder::default().build().initialize_oracle_prices().initialize_all_vaults().execute_with(|| {
// 			random_option_configs.iter().for_each(|option_config|{

// 				let option_config = OptionsConfigBuilder::default().base_asset_id(option_config.0).base_asset_strike_price(option_config.1).build();

// 				match trait_create_option(Origin::signed(ADMIN), option_config.clone()) {
// 					Ok(option_id) => {
// 						assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));

// 						System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
// 							option_id,
// 							option_config,
// 						}));
// 					},
// 					Err(error) => {
// 						assert_eq!(error, DispatchError::from(Error::<MockRuntime>::OptionAssetVaultsDoNotExist));
// 					}
// 				};
// 			})
// 		});
// 	}
// }
