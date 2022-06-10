use crate::mock::runtime::{
	Assets, Balance, Balances, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions, Vault,
};

use crate::mock::{accounts::*, assets::*};

use crate::{
	pallet::{self, OptionHashToOptionId, Sellers},
	tests::*,
};

use composable_traits::vault::CapabilityVault;
use composable_traits::{
	tokenized_options::TokenizedOptions as TokenizedOptionsTrait, vault::Vault as VaultTrait,
};
use frame_support::{assert_err, assert_noop, assert_ok, traits::fungibles::Inspect};

use frame_system::ensure_signed;
use sp_core::{sr25519::Public, H256};
use sp_runtime::ArithmeticError;

// ----------------------------------------------------------------------------------------------------
//		Sell Options Tests
// ----------------------------------------------------------------------------------------------------

pub fn sell_option_success_checks(
	option_hash: H256,
	option_config: OptionConfig<AssetId, Balance, Moment>,
	option_amount: Balance,
	who: Public,
) {
	// Get info before extrinsic for checks
	let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

	// Different behaviors based on Call or Put option
	let (asset_id, asset_amount) = match option_config.option_type {
		OptionType::Call => {
			(option_config.base_asset_id, option_config.base_asset_amount_per_option)
		},
		OptionType::Put => (option_config.quote_asset_id, option_config.base_asset_strike_price),
	};

	let asset_amount = asset_amount * option_amount;
	let vault_id = AssetToVault::<MockRuntime>::get(asset_id).unwrap();
	let lp_token_id = <Vault as VaultTrait>::lp_asset_id(&vault_id).unwrap();
	let protocol_account = TokenizedOptions::account_id(asset_id);
	let shares_amount =
		<Vault as VaultTrait>::calculate_lp_tokens_to_mint(&vault_id, asset_amount).unwrap();

	let initial_issuance_seller =
		OptionIdToOption::<MockRuntime>::get(option_id).unwrap().total_issuance_seller;
	let initial_user_balance = Assets::balance(asset_id, &who);
	let initial_vault_balance = Assets::balance(asset_id, &Vault::account_id(&vault_id));
	let initial_user_position = Sellers::<MockRuntime>::try_get(option_id, who).unwrap_or_default();

	// Call extrinsic
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
	assert_eq!(Assets::balance(asset_id, &who), initial_user_balance - asset_amount);

	// Check vault balance after sale is correct
	assert_eq!(
		Assets::balance(asset_id, &Vault::account_id(&vault_id)),
		initial_vault_balance + asset_amount
	);

	// Check protocol owns all the issuance of lp_token
	assert_eq!(
		Assets::balance(lp_token_id, &protocol_account),
		Assets::total_issuance(lp_token_id)
	);

	// Check position is updated correctly
	let updated_user_position = Sellers::<MockRuntime>::try_get(option_id, who).unwrap_or_default();

	assert_eq!(
		updated_user_position.option_amount,
		initial_user_position.option_amount + option_amount,
	);
	assert_eq!(
		updated_user_position.shares_amount,
		initial_user_position.shares_amount + shares_amount,
	);

	// Check position is updated correctly
	let updated_issuance_seller = OptionIdToOption::<MockRuntime>::try_get(option_id)
		.unwrap()
		.total_issuance_seller;

	assert_eq!(updated_issuance_seller, initial_issuance_seller + option_amount)
}

#[test]
fn test_sell_option_with_initialization_success() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 1 * 10u128.pow(12)),
			(BOB, USDC, 50000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.execute_with(|| {
			// Get BTC and USDC vault config
			let btc_vault_config = VaultConfigBuilder::default().build();
			let usdc_vault_config = VaultConfigBuilder::default().asset_id(USDC).build();

			// Create BTC and USDC vaults
			assert_ok!(TokenizedOptions::create_asset_vault(
				Origin::signed(ADMIN),
				btc_vault_config
			));

			assert_ok!(TokenizedOptions::create_asset_vault(
				Origin::signed(ADMIN),
				usdc_vault_config
			));

			// Create default BTC option
			let option_config = OptionsConfigBuilder::default().build();

			assert_ok!(TokenizedOptions::create_option(
				Origin::signed(ADMIN),
				option_config.clone()
			));

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
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
		.initialize_balances(Vec::from([
			(BOB, BTC, 7 * 10u128.pow(12)),
			(BOB, USDC, 350000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_amount = 7u128; // Same as BOB's balance

			sell_option_success_checks(option_hash, option_config, option_amount, BOB);
		});
}

#[test]
fn test_sell_option_update_position() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 5 * 10u128.pow(12)),
			(BOB, USDC, 250000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
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
			(ALICE, USDC, 500000 * 10u128.pow(12)),
			(BOB, USDC, 350000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
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
		.initialize_balances(Vec::from([
			(BOB, BTC, 1 * 10u128.pow(12)),
			(BOB, USDC, 50000 * 10u128.pow(12)),
		]))
		.build()
		.execute_with(|| {
			assert_noop!(
				// 10000000000005u128 it's a meaningless number
				TokenizedOptions::sell_option(
					Origin::signed(BOB),
					1u128,
					AssetId(10000000000005u128)
				),
				Error::<MockRuntime>::OptionDoesNotExists
			);
		});
}

#[test]
fn test_sell_option_error_not_into_deposit_window() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 5 * 10u128.pow(12)),
			(BOB, USDC, 250000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			// Default config deposit window is between timestamp 0 <= x < 3000.
			// Each block takes 1 second, so on block 3 should already be out of window
			run_to_block(3);

			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), 8u128, option_id),
				Error::<MockRuntime>::NotIntoDepositWindow
			);
		});
}

#[test]
fn test_sell_option_error_user_has_not_enough_funds() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 5 * 10u128.pow(12)),
			(BOB, USDC, 250000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), 8u128, option_id),
				Error::<MockRuntime>::UserHasNotEnoughFundsToDeposit
			);
		});
}

#[test]
fn test_sell_option_error_user_has_not_enough_funds_update_position() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 5 * 10u128.pow(12)),
			(BOB, USDC, 250000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			assert_ok!(TokenizedOptions::sell_option(Origin::signed(BOB), 3u128, option_id));

			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), 3u128, option_id),
				Error::<MockRuntime>::UserHasNotEnoughFundsToDeposit
			);
		});
}

#[test]
fn test_sell_option_error_cannot_sell_zero_options() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 5 * 10u128.pow(12)),
			(BOB, USDC, 250000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), 0u128, option_id),
				Error::<MockRuntime>::CannotPassZeroOptionAmount
			);
		});
}

#[test]
fn test_sell_option_error_overflow_asset_amount() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 5 * 10u128.pow(12)),
			(BOB, USDC, 250000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			// Balance: u128 contains until ~4 * 10^38. Considering 12 decimals,
			// the asset_amount to transfer should overflow with option amount > 3 * 10^26.
			// It works until 3 * 10^26.
			let option_amount = 4 * 10u128.pow(26);

			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), option_amount, option_id),
				ArithmeticError::Overflow
			);
		});
}

#[test]
fn test_sell_option_error_deposits_not_allowed() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 5 * 10u128.pow(12)),
			(BOB, USDC, 250000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			let vault_id = match option_config.option_type {
				OptionType::Call => {
					TokenizedOptions::asset_id_to_vault_id(option_config.base_asset_id).unwrap()
				},
				OptionType::Put => {
					TokenizedOptions::asset_id_to_vault_id(option_config.quote_asset_id).unwrap()
				},
			};

			assert_ok!(<Vault as CapabilityVault>::stop_deposits(&vault_id));

			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), 5u128, option_id),
				Error::<MockRuntime>::VaultDepositNotAllowed
			);
		});
}

#[test]
fn test_sell_option_error_deposits_not_allowed_update_position() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(BOB, BTC, 5 * 10u128.pow(12)),
			(BOB, USDC, 250000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			assert_ok!(TokenizedOptions::sell_option(Origin::signed(BOB), 3u128, option_id));

			let vault_id = match option_config.option_type {
				OptionType::Call => {
					TokenizedOptions::asset_id_to_vault_id(option_config.base_asset_id).unwrap()
				},
				OptionType::Put => {
					TokenizedOptions::asset_id_to_vault_id(option_config.quote_asset_id).unwrap()
				},
			};

			assert_ok!(<Vault as CapabilityVault>::stop_deposits(&vault_id));

			assert_noop!(
				TokenizedOptions::sell_option(Origin::signed(BOB), 2u128, option_id),
				Error::<MockRuntime>::VaultDepositNotAllowed
			);
		});
}

#[test]
fn test_sell_option_shares_calculation_with_vault_value_accrual_success() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 10 * 10u128.pow(12)),
			(BOB, BTC, 10 * 10u128.pow(12)),
			(CHARLIE, BTC, 10 * 10u128.pow(12)),
			(ALICE, USDC, 500000 * 10u128.pow(12)),
			(BOB, USDC, 500000 * 10u128.pow(12)),
			(CHARLIE, USDC, 500000 * 10u128.pow(12)),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			let option_config = OptionsConfigBuilder::default().build();

			let option_hash = TokenizedOptions::generate_id(
				option_config.base_asset_id,
				option_config.quote_asset_id,
				option_config.base_asset_strike_price,
				option_config.quote_asset_strike_price,
				option_config.option_type,
				option_config.expiring_date,
				option_config.exercise_type,
			);
			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let alice_option_amount = 5u128;
			sell_option_success_checks(
				option_hash,
				option_config.clone(),
				alice_option_amount,
				ALICE,
			);

			// Add 1 BTC to the vault to simulate vault value accrual
			let vault_id = AssetToVault::<MockRuntime>::get(option_config.base_asset_id).unwrap();
			let vault_account = Vault::account_id(&vault_id);
			assert_ok!(Assets::mint_into(
				Origin::signed(ADMIN),
				option_config.base_asset_id,
				vault_account,
				1 * 10u128.pow(12),
			));

			let bob_option_amount = 5u128;
			sell_option_success_checks(option_hash, option_config.clone(), bob_option_amount, BOB);

			// Remove 2 BTC from the vault to simulate vault value loss
			assert_ok!(Assets::burn_from(
				Origin::signed(ADMIN),
				option_config.base_asset_id,
				vault_account,
				2 * 10u128.pow(12),
			));

			let charlie_option_amount = 5u128;
			sell_option_success_checks(
				option_hash,
				option_config.clone(),
				charlie_option_amount,
				CHARLIE,
			);

			// Remove 2 BTC from the vault to simulate vault value loss
			assert_ok!(Assets::burn_from(
				Origin::signed(ADMIN),
				option_config.base_asset_id,
				vault_account,
				2 * 10u128.pow(12),
			));

			let alice_option_amount = 5u128;
			sell_option_success_checks(
				option_hash,
				option_config.clone(),
				alice_option_amount,
				ALICE,
			);

			assert_ok!(Assets::mint_into(
				Origin::signed(ADMIN),
				option_config.base_asset_id,
				vault_account,
				4 * 10u128.pow(12),
			));

			let bob_option_amount = 5u128;
			sell_option_success_checks(option_hash, option_config.clone(), bob_option_amount, BOB);

			assert_ok!(Assets::mint_into(
				Origin::signed(ADMIN),
				option_config.base_asset_id,
				vault_account,
				1 * 10u128.pow(12),
			));

			let charlie_option_amount = 5u128;
			sell_option_success_checks(option_hash, option_config, charlie_option_amount, CHARLIE);
		});
}
