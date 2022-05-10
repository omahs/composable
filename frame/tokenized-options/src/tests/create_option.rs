use crate::mock::runtime::{
	accounts::*, AssetId, Balance, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions,
};
use crate::tests::*;
use crate::{pallet, OptionIdToOption};
use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use frame_system::ensure_signed;

// Simulate exstrinsic call `create_option`, but returning values
fn trait_create_option(
	origin: Origin,
	option_config: OptionConfig<AssetId, Balance, Moment>,
) -> AssetId {
	let _account_id = ensure_signed(origin).unwrap();

	let option_id =
		<TokenizedOptions as TokenizedOptionsTrait>::create_option(option_config.clone()).unwrap();

	TokenizedOptions::deposit_event(pallet::Event::CreatedOption { option_id, option_config });

	option_id
}

// ----------------------------------------------------------------------------------------------------
//		Create Options Tests
// ----------------------------------------------------------------------------------------------------
#[test]
fn test_create_option_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		// Get default option
		let option_config = OptionsConfigBuilder::default().build();

		// Create option and get option id
		let option_id = trait_create_option(Origin::signed(ADMIN), option_config.clone());

		// Check option has been created
		assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
			option_id,
			option_config,
		}));
	});
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(10))]
	#[test]
	fn proptest_create_option(market_prices in generate_markets()) {
		ExtBuilder::default().build().execute_with(|| {
			let option_configs: Vec<OptionConfig<AssetId, Balance, Moment>> = market_prices.iter().map(|&(asset, strike_price)| {
				OptionsConfigBuilder::default().base_asset_id(asset).base_asset_strike_price(strike_price).build()
			}).collect();

			option_configs.iter().for_each(|option_config|{
				let option_id = trait_create_option(Origin::signed(ADMIN), option_config.clone());

				assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));

				System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
					option_id,
					option_config: option_config.clone(),
				}));
			})
		});
	}
}

// TODO: add tests for creating same option
