use crate::mock::runtime::{
	Assets, Balance, Balances, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions, Vault,
};

use crate::mock::{accounts::*, assets::*};

use crate::{
	pallet::{self, OptionHashToOptionId, Sellers},
	tests::{
		buy_option::buy_option_success_checks,
		delete_sell_option::delete_sell_option_success_checks,
		sell_option::sell_option_success_checks, *,
	},
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
//		Settle Options Tests
// ----------------------------------------------------------------------------------------------------

pub fn settle_options_success_checks(whos: &Vec<(Public, Balance)>) {
	// let now = Timestamp::get();

	// let option_n = OptionIdToOption::<MockRuntime>::iter().count();

	for (option_id, option) in OptionIdToOption::<MockRuntime>::iter() {
		// If option has not been sold, skip the check
		// do_settle_option panics when trying to withdraw 0 from the vault
		if option.total_issuance_seller == 0 {
			continue;
		}

		let base_asset_spot_price = get_oracle_price(option.base_asset_id, UNIT);

		let (asset_id, collateral_for_buyers) = match option.option_type {
			OptionType::Call => (
				option.base_asset_id,
				TokenizedOptions::call_option_collateral_amount(base_asset_spot_price, &option)
					.unwrap(),
			),
			OptionType::Put => (
				option.quote_asset_id,
				TokenizedOptions::put_option_collateral_amount(base_asset_spot_price, &option)
					.unwrap(),
			),
		};

		let total_collateral_for_buyers = collateral_for_buyers * Assets::total_issuance(option_id);
		let protocol_account = TokenizedOptions::account_id(asset_id);
		let initial_protocol_balance = Assets::balance(asset_id, &protocol_account);
		let vault_id = AssetToVault::<MockRuntime>::get(asset_id).unwrap();
		let initial_vault_balance = Assets::balance(asset_id, &Vault::account_id(&vault_id));
		let shares_amount =
			Vault::calculate_lp_tokens_from_asset_amount(&vault_id, collateral_for_buyers).unwrap();
		let mut initial_user_positions: Vec<(Public, Balance)> = vec![];
		for (user, _) in whos {
			initial_user_positions.push((
				*user,
				Sellers::<MockRuntime>::try_get(option_id, user)
					.unwrap_or_default()
					.shares_amount,
			));
		}

		// Call function
		assert_ok!(TokenizedOptions::do_settle_option(option_id, &option));

		// // Check correct event
		// System::assert_last_event(Event::TokenizedOptions(pallet::Event::SettleOptions {
		// 	timestamp: now,
		// }));

		let updated_protocol_balance = Assets::balance(asset_id, &protocol_account);
		let updated_vault_balance = Assets::balance(asset_id, &Vault::account_id(&vault_id));
		let mut updated_user_positions: Vec<(Public, Balance)> = vec![];
		for (user, _) in whos {
			updated_user_positions.push((
				*user,
				Sellers::<MockRuntime>::try_get(option_id, user)
					.unwrap_or_default()
					.shares_amount,
			));
		}

		assert_eq!(
			updated_protocol_balance,
			initial_protocol_balance + total_collateral_for_buyers,
		);

		assert_eq!(updated_vault_balance, initial_vault_balance - total_collateral_for_buyers);

		for (i, (user, user_option_amount)) in whos.iter().enumerate() {
			assert_eq!(initial_user_positions[i].0, *user);
			assert_eq!(updated_user_positions[i].0, *user);
			assert_eq!(
				updated_user_positions[i].1,
				initial_user_positions[i].1 - shares_amount * user_option_amount
			);
		}
	}
}

#[test]
fn test_settle_options_with_initialization_success() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 10 * UNIT),
			(ALICE, USDC, 500000 * UNIT),
			(BOB, BTC, 10 * UNIT),
			(BOB, USDC, 500000 * UNIT),
			(CHARLIE, BTC, 10 * UNIT),
			(CHARLIE, USDC, 500000 * UNIT),
			(DAVE, BTC, 10 * UNIT),
			(DAVE, USDC, 500000 * UNIT),
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

			// Sell option and make checks
			let alice_option_amount = 5u128;
			let bob_option_amount = 4u128;
			let charlie_option_amount = 3u128;
			let dave_option_amount = 6u128;

			sell_option_success_checks(
				option_hash,
				option_config.clone(),
				alice_option_amount,
				ALICE,
			);

			sell_option_success_checks(option_hash, option_config.clone(), bob_option_amount, BOB);

			// Go to purchase window
			run_to_block(3);

			// Buy option
			buy_option_success_checks(
				option_hash,
				option_config.clone(),
				charlie_option_amount,
				CHARLIE,
			);

			buy_option_success_checks(option_hash, option_config.clone(), dave_option_amount, DAVE);

			// BTC price moves from 50k to 55k, buyers are in profit
			set_oracle_price(option_config.base_asset_id, 55000u128 * UNIT);

			// Go to exercise window (option has expired so settlement can start)
			run_to_block(6);

			let whos = vec![(ALICE, alice_option_amount), (BOB, bob_option_amount)];
			// Settle options
			settle_options_success_checks(&whos);
		});
}

#[test]
fn test_settle_options_success_multiple_options() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 30 * UNIT),
			(ALICE, USDC, 1500000 * UNIT),
			(BOB, BTC, 30 * UNIT),
			(BOB, USDC, 1500000 * UNIT),
			(CHARLIE, BTC, 30 * UNIT),
			(CHARLIE, USDC, 1500000 * UNIT),
			(DAVE, BTC, 30 * UNIT),
			(DAVE, USDC, 1500000 * UNIT),
		]))
		.build()
		.initialize_oracle_prices()
		.initialize_all_vaults()
		.initialize_all_options()
		.execute_with(|| {
			// Create default BTC option
			let option_config_1 = OptionsConfigBuilder::default().build();
			let option_config_2 = OptionsConfigBuilder::default()
				.base_asset_strike_price(55000u128 * UNIT)
				.build();

			let option_hash_1 = TokenizedOptions::generate_id(
				option_config_1.base_asset_id,
				option_config_1.quote_asset_id,
				option_config_1.base_asset_strike_price,
				option_config_1.quote_asset_strike_price,
				option_config_1.option_type,
				option_config_1.expiring_date,
				option_config_1.exercise_type,
			);

			let option_hash_2 = TokenizedOptions::generate_id(
				option_config_2.base_asset_id,
				option_config_2.quote_asset_id,
				option_config_2.base_asset_strike_price,
				option_config_2.quote_asset_strike_price,
				option_config_2.option_type,
				option_config_2.expiring_date,
				option_config_2.exercise_type,
			);

			// Sell option and make checks
			let alice_option_amount = 7u128;
			let bob_option_amount = 9u128;
			let charlie_option_amount = 1u128;
			let dave_option_amount = 15u128;

			sell_option_success_checks(
				option_hash_1,
				option_config_1.clone(),
				alice_option_amount,
				ALICE,
			);

			sell_option_success_checks(
				option_hash_1,
				option_config_1.clone(),
				bob_option_amount,
				BOB,
			);

			sell_option_success_checks(
				option_hash_2,
				option_config_2.clone(),
				alice_option_amount,
				ALICE,
			);

			sell_option_success_checks(
				option_hash_2,
				option_config_2.clone(),
				bob_option_amount,
				BOB,
			);

			// Go to purchase window
			run_to_block(3);

			// Buy option
			buy_option_success_checks(
				option_hash_1,
				option_config_1.clone(),
				charlie_option_amount,
				CHARLIE,
			);

			buy_option_success_checks(
				option_hash_1,
				option_config_1.clone(),
				dave_option_amount,
				DAVE,
			);

			// Buy option
			buy_option_success_checks(
				option_hash_2,
				option_config_2.clone(),
				charlie_option_amount,
				CHARLIE,
			);

			buy_option_success_checks(
				option_hash_2,
				option_config_2.clone(),
				dave_option_amount,
				DAVE,
			);

			// BTC price moves from 50k to 60k, all buyers are in profit
			set_oracle_price(option_config_1.base_asset_id, 60000u128 * UNIT);
			set_oracle_price(PICA, 2u128 * UNIT);

			// Go to exercise window (option has expired so settlement can start)
			run_to_block(6);

			let whos = vec![(ALICE, alice_option_amount), (BOB, bob_option_amount)];
			// Settle options
			settle_options_success_checks(&whos);
		});
}
