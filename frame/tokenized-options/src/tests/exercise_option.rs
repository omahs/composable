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
//		Exercise Options Tests
// ----------------------------------------------------------------------------------------------------
pub fn exercise_option_success_checks(
	option_hash: H256,
	option_config: OptionConfig<AssetId, Balance, Moment>,
	option_amount: Balance,
	who: Public,
) {
	// Get info before extrinsic for checks
	let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

	// // Different behaviors based on Call or Put option
	// let (asset_id, asset_amount) = match option_config.option_type {
	// 	OptionType::Call => {
	// 		(option_config.base_asset_id, option_config.base_asset_amount_per_option)
	// 	},
	// 	OptionType::Put => (option_config.quote_asset_id, option_config.base_asset_strike_price),
	// };

	// let asset_amount = asset_amount * option_amount;
	// let vault_id = AssetToVault::<MockRuntime>::get(asset_id).unwrap();
	// let lp_token_id = <Vault as VaultTrait>::lp_asset_id(&vault_id).unwrap();
	// let protocol_account = TokenizedOptions::account_id(asset_id);
	// let shares_amount =
	// 	<Vault as VaultTrait>::calculate_lp_tokens_to_mint(&vault_id, asset_amount).unwrap();

	// let initial_issuance_seller =
	// 	OptionIdToOption::<MockRuntime>::get(option_id).unwrap().total_issuance_seller;
	// let initial_user_balance = Assets::balance(asset_id, &who);
	// let initial_vault_balance = Assets::balance(asset_id, &Vault::account_id(&vault_id));
	// let initial_user_position = Sellers::<MockRuntime>::try_get(option_id, who).unwrap_or_default();

	// Call extrinsic
	assert_ok!(TokenizedOptions::exercise_option(Origin::signed(who), option_amount, option_id));

	// Check correct event
	System::assert_last_event(Event::TokenizedOptions(pallet::Event::ExerciseOption {
		user: who,
		option_amount,
		option_id,
	}));

	// // Check seller position is saved
	// assert!(Sellers::<MockRuntime>::contains_key(option_id, who));

	// // Check seller balance after sale is empty
	// assert_eq!(Assets::balance(asset_id, &who), initial_user_balance - asset_amount);

	// // Check vault balance after sale is correct
	// assert_eq!(
	// 	Assets::balance(asset_id, &Vault::account_id(&vault_id)),
	// 	initial_vault_balance + asset_amount
	// );

	// // Check protocol owns all the issuance of lp_token
	// assert_eq!(
	// 	Assets::balance(lp_token_id, &protocol_account),
	// 	Assets::total_issuance(lp_token_id)
	// );

	// // Check position is updated correctly
	// let updated_user_position = Sellers::<MockRuntime>::try_get(option_id, who).unwrap_or_default();

	// assert_eq!(
	// 	updated_user_position.option_amount,
	// 	initial_user_position.option_amount + option_amount,
	// );
	// assert_eq!(
	// 	updated_user_position.shares_amount,
	// 	initial_user_position.shares_amount + shares_amount,
	// );

	// // Check position is updated correctly
	// let updated_issuance_seller = OptionIdToOption::<MockRuntime>::try_get(option_id)
	// 	.unwrap()
	// 	.total_issuance_seller;

	// assert_eq!(updated_issuance_seller, initial_issuance_seller + option_amount)
}

#[test]
fn test_exercise_option_with_initialization_success() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([
			(ALICE, BTC, 1 * 10u128.pow(12)),
			(ALICE, USDC, 50000 * 10u128.pow(12)),
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

			// Sell option and make checks
			let option_amount = 1u128;
			sell_option_success_checks(option_hash, option_config.clone(), option_amount, BOB);

			// Go to purchase window
			run_to_block(3);

			// Buy option
			buy_option_success_checks(option_hash, option_config.clone(), option_amount, ALICE);

			// BTC price moves from 50k to 55k
			set_oracle_price(option_config.base_asset_id, 55000u128 * 10u128.pow(12));

			// Go to exercise window
			run_to_block(7);

			// Exercise option
			exercise_option_success_checks(option_hash, option_config, option_amount, ALICE);
		});
}
