use crate::mock::runtime::{
	Assets, Balance, Balances, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions, Vault,
};

use crate::mock::{accounts::*, assets::*};

use crate::{
	pallet::{self, OptionHashToOptionId, Sellers},
	tests::{
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
//		Sell Options Tests
// ----------------------------------------------------------------------------------------------------

pub fn buy_option_success_checks(
	option_hash: H256,
	option_config: OptionConfig<AssetId, Balance, Moment>,
	option_amount: Balance,
	who: Public,
) {
	// Get info before extrinsic for checks
	let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();
	let asset_id = option_config.quote_asset_id;

	let protocol_account = TokenizedOptions::account_id(asset_id);

	let initial_issuance_buyer =
		OptionIdToOption::<MockRuntime>::get(option_id).unwrap().total_issuance_buyer;
	let initial_user_balance = Assets::balance(asset_id, &who);
	let initial_protocol_balance = Assets::balance(asset_id, &protocol_account);

	// Call exstrinsic
	assert_ok!(TokenizedOptions::buy_option(Origin::signed(who), option_amount, option_id));

	// Check correct event
	System::assert_last_event(Event::TokenizedOptions(pallet::Event::BuyOption {
		buyer: who,
		option_amount,
		option_id,
	}));

	let option_premium = TokenizedOptions::fake_option_price().unwrap();

	// Check buyer balance after sale has premium subtracted
	assert_eq!(Assets::balance(asset_id, &who), initial_user_balance - option_premium);

	// Check protocol balance after purchase is correct
	assert_eq!(
		Assets::balance(asset_id, &protocol_account),
		initial_protocol_balance + option_premium
	);

	// Check user owns the correct issuance of option token
	assert_eq!(Assets::balance(option_id, &who), option_amount);

	// Check position is updated correctly
	let updated_issuance_buyer = OptionIdToOption::<MockRuntime>::try_get(option_id)
		.unwrap()
		.total_issuance_buyer;

	assert_eq!(updated_issuance_buyer, initial_issuance_buyer + option_amount)
}

#[test]
fn test_buy_option_with_initialization_success() {
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
			buy_option_success_checks(option_hash, option_config, option_amount, ALICE);
		});
}
