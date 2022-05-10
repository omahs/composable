// use crate::mock::runtime::{accounts::*, AssetId, Balance, Origin, TokenizedOptions};
// use crate::pallet::{self};
// use crate::tests::*;

// use composable_traits::tokenized_options::TokenizedOptions;
// use frame_system::ensure_signed;

// Simulate exstrinsic call `sell_option`, but returning values
// fn trait_sell_option(_origin: Origin, _amount: Balance, _option_id: AssetId) -> () {
// 	let account_id = ensure_signed(_origin).unwrap();

// 	// Not yet correctly implemented
// 	<TokenizedOptions>::sell_option(&account_id, _amount, _option_id).unwrap();

// 	TokenizedOptions::deposit_event(pallet::Event::SellOption {
// 		seller: account_id,
// 		option_amount: _amount,
// 		option_id: _option_id,
// 	});

// 	()
// }

// ----------------------------------------------------------------------------------------------------
//		Sell/Buy Options Tests
// ----------------------------------------------------------------------------------------------------
// #[test]
// fn test_sell_option_and_emit_event() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let option_btc = OptionsBuilder::default().build();

// 		assert_ok!(TokenizedOptions::create_option(Origin::signed(ADMIN), option_btc.clone()));

// 		assert!(OptionIdToOption::<MockRuntime>::contains_key(100_000_000_001u128));

// 		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOption {
// 			option_id: 100_000_000_001u128,
// 			option: option_btc.clone(),
// 		}));

// 		assert_ok!(TokenizedOptions::sell_option(
// 			Origin::signed(ADMIN),
// 			1u128,
// 			100_000_000_001u128
// 		));

// 		System::assert_last_event(Event::TokenizedOptions(pallet::Event::SellOption {
// 			option_id: 100_000_000_001u128,
// 			who: ADMIN,
// 			amount: 1u128,
// 		}));
// 	});
// }

// proptest! {
// 	#![proptest_config(ProptestConfig::with_cases(100))]
// 	#[test]
// 	fn proptest_sell_option(blockchain_state in generate_blockchain_state()) {

// 		let balances: Vec<(AccountId, AssetId, Balance)> = blockchain_state.into_iter().map(|(account, asset, balance, price)| (account, asset, balance)).collect();

// 		ExtBuilder::default().init_balances(balances.clone()).build().execute_with(|| {

// 			// let options: Vec<OptionToken<AssetId, Balance>> = balances.iter().map(|&(_, asset, price)| {
// 			// 	OptionsBuilder::default().base_asset_id(asset).strike_price(price).build()
// 			// }).collect();

// 			// options.iter().for_each(|option|{
// 			// 	assert_ok!(TokenizedOptions::create_option_with_vault(Origin::signed(ADMIN), option.clone()));
// 			// 	let (option_id, vault_id) =
// 			// 	trait_create_option_with_vault(Origin::signed(ADMIN), &option);

// 			// 	assert!(OptionIdToOption::<MockRuntime>::contains_key(option_id));
// 			// 	assert!(OptionToVault::<MockRuntime>::contains_key(option_id));

// 			// 	System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedOptionVault {
// 			// 		option_id,
// 			// 		vault_id,
// 			// 		option: option.clone(),
// 			// 	}));
// 			// })
// 		});
// 	}
// }
