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
		sell_option::sell_option_success_checks, settle_options::settle_options_success_checks, *,
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
//		Withdraw Collateral Tests
// ----------------------------------------------------------------------------------------------------
pub fn withdraw_collateral_success_checks(option_id: AssetId, who: Public) {
	let option = OptionIdToOption::<MockRuntime>::get(option_id).unwrap();

	// Different behaviors based on Call or Put option
	let asset_id = match option.option_type {
		OptionType::Call => option.base_asset_id,
		OptionType::Put => option.quote_asset_id,
	};

	// ---------------------------
	// |  Data before extrinsic  |
	// ---------------------------
	let vault_id = AssetToVault::<MockRuntime>::get(asset_id).unwrap();
	let lp_token_id = <Vault as VaultTrait>::lp_asset_id(&vault_id).unwrap();
	let protocol_account = TokenizedOptions::account_id(asset_id);
	let initial_user_balance = Assets::balance(asset_id, &who);
	let initial_vault_balance = Assets::balance(asset_id, &Vault::account_id(&vault_id));
	let initial_user_position = Sellers::<MockRuntime>::try_get(option_id, who).unwrap_or_default();
	let asset_amount = Vault::lp_share_value(&vault_id, initial_user_position.shares_amount);

	// Call extrinsic and check event
	assert_ok!(TokenizedOptions::withdraw_collateral(Origin::signed(who), option_id));

	// Check correct event
	System::assert_last_event(Event::TokenizedOptions(pallet::Event::WithdrawCollateral {
		user: who,
		option_id,
	}));

	// --------------------------
	// |  Data after extrinsic  |
	// --------------------------

	// Check seller position has been deleted
	assert!(!Sellers::<MockRuntime>::contains_key(option_id, who));

	// Check seller balance after sale is empty
	assert_eq!(Assets::balance(asset_id, &who), initial_user_balance + asset_amount);

	// // Check vault balance after sale is correct
	assert_eq!(
		Assets::balance(asset_id, &Vault::account_id(&vault_id)),
		initial_vault_balance - asset_amount
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
		initial_user_position.option_amount - option_amount,
	);
	assert_eq!(
		updated_user_position.shares_amount,
		initial_user_position.shares_amount - shares_amount,
	);

	// Check position is updated correctly
	let updated_issuance_seller = OptionIdToOption::<MockRuntime>::try_get(option_id)
		.unwrap()
		.total_issuance_seller;

	assert_eq!(updated_issuance_seller, initial_issuance_seller - option_amount)
}

#[test]
fn test_withdraw_collateral_call_with_initialization_success() {
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
			let option_config =
				OptionsConfigBuilder::default().option_type(OptionType::Call).build();

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
			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			// Sell option and make checks
			let alice_option_amount = 5u128;
			let bob_option_amount = 4u128;
			let charlie_option_amount = 3u128;
			let dave_option_amount = 6u128;

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(ALICE),
				alice_option_amount,
				option_id
			));

			assert_ok!(TokenizedOptions::sell_option(
				Origin::signed(BOB),
				bob_option_amount,
				option_id
			));

			// Go to purchase window
			run_to_block(3);

			// Buy option
			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id
			));

			assert_ok!(TokenizedOptions::buy_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id
			));

			// BTC price moves from 50k to 55k, buyers are in profit
			set_oracle_price(option_config.base_asset_id, 55000u128 * UNIT);

			// Go to exercise window (option has expired so settlement can start)
			run_to_block(6);

			// Settle options
			assert_ok!(TokenizedOptions::settle_options());

			assert_ok!(TokenizedOptions::exercise_option(
				Origin::signed(CHARLIE),
				charlie_option_amount,
				option_id
			));
			assert_ok!(TokenizedOptions::exercise_option(
				Origin::signed(DAVE),
				dave_option_amount,
				option_id
			));

			withdraw_collateral_success_checks(option_id, CHARLIE);
		});
}
