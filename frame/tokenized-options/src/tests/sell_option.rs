use crate::mock::currency::defs::*;
use crate::mock::runtime::{
	accounts::*, AssetId, Balance, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions,
};

use crate::pallet::{self, OptionHashToOptionId};
use crate::tests::*;

use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use frame_system::ensure_signed;

// ----------------------------------------------------------------------------------------------------
//		Sell Options Tests
// ----------------------------------------------------------------------------------------------------
#[test]
fn test_sell_option_and_emit_event() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([(ADMIN, BTC::ID, BTC::units(1))]))
		.build()
		.initialize_oracle_prices()
		.execute_with(|| {
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

			assert!(OptionHashToOptionId::<MockRuntime>::contains_key(option_hash));

			let option_id = OptionHashToOptionId::<MockRuntime>::get(option_hash).unwrap();

			assert_ok!(TokenizedOptions::sell_option(Origin::signed(ADMIN), 1u128, option_id));

			System::assert_last_event(Event::TokenizedOptions(pallet::Event::SellOption {
				seller: ADMIN,
				option_amount: 1u128,
				option_id,
			}));
		});
}

#[test]
fn test_sell_option_and_emit_event2() {
	ExtBuilder::default()
		.initialize_balances(Vec::from([(ADMIN, BTC::ID, BTC::units(1))]))
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

			assert_ok!(TokenizedOptions::sell_option(Origin::signed(ADMIN), 1u128, option_id));

			System::assert_last_event(Event::TokenizedOptions(pallet::Event::SellOption {
				seller: ADMIN,
				option_amount: 1u128,
				option_id,
			}));
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
