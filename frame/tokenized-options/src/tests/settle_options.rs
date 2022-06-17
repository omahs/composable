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

pub fn settle_options_success_checks(
	option_hash: H256,
	option_config: OptionConfig<AssetId, Balance, Moment>,
	option_amount: Balance,
	who: Public,
) {
	// Get info before extrinsic for checks
	let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

	// Call extrinsic
	let now = Timestamp::get();

	assert_ok!(<TokenizedOptions as TokenizedOptionsTrait>::settle_options(now));

	// Check correct event
	System::assert_last_event(Event::TokenizedOptions(pallet::Event::SettleOptions {
		timestamp: now,
	}));
}

#[test]
fn test_settle_options_with_initialization_success() {
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

			// Go to exercise window (option has expired so settlement can start)
			run_to_block(6);

			// Settle options
			settle_options_success_checks(option_hash, option_config, option_amount, ALICE);
		});
}
