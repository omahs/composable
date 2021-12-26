use crate::runtime::*;
use composable_tests_helpers::test::currency::MockCurrencyId;
use composable_traits::{defi::{Sell, CurrencyPair, Take}, auction::{AuctionStepFunction, LinearDecrease}};
use frame_system::pallet_prelude::BlockNumberFor;
use orml_traits::MultiReservableCurrency;
use pallet_balances;

use frame_support::{
	assert_noop, assert_ok,
	traits::{fungibles::{Inspect, Mutate}, Hooks}, assert_err, dispatch::DispatchErrorWithPostInfo,
};

pub fn new_test_externalities() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
    pallet_balances::GenesisConfig::<Runtime> { balances: vec![], }
    .assimilate_storage(&mut storage).unwrap();

    let mut externatlities = sp_io::TestExternalities::new(storage);
	externatlities.execute_with(|| {
		System::set_block_number(42);
		Timestamp::set_timestamp(System::block_number() * MILLISECS_PER_BLOCK);
	});
	externatlities
}


#[test]
fn flow_with_immediate_exact_buy() {
	new_test_externalities().execute_with(|| {
		let a = 1_000_000;
		let b = 10;
		Tokens::mint_into(MockCurrencyId::USDT, &BOB, a).unwrap();
		Tokens::mint_into(MockCurrencyId::BTC, &ALICE, b).unwrap();
        let seller = AccountId::from_raw(ALICE.0);
        let buyer = AccountId::from_raw(BOB.0);
        let sell_amount = 1;
        let take_amount = 1000;
        let sell = Sell::new(MockCurrencyId::BTC, MockCurrencyId::USDT, sell_amount, take_amount); 
        let invalid = crate::OrdersIndex::<Runtime>::get();
        let configuration = AuctionStepFunction::LinearDecrease(LinearDecrease {
            total : 42,
         });
        let not_reserved = Assets::reserved_balance(MockCurrencyId::BTC, &ALICE);
        DutchAuction::ask(Origin::signed(seller), sell, configuration).unwrap();
        let reserved = Assets::reserved_balance(MockCurrencyId::BTC, &ALICE);
        assert!(not_reserved < reserved && reserved == sell_amount);
        let order_id = crate::OrdersIndex::<Runtime>::get();
        assert_ne!(invalid, order_id);
        let result = DutchAuction::take(Origin::signed(buyer), order_id, Take::new(1, 999));
        assert!(!result.is_ok());     
        let not_reserved = Assets::reserved_balance(MockCurrencyId::USDT, &BOB);        
        let result = DutchAuction::take(Origin::signed(buyer), order_id, Take::new(1, 1000));
        let reserved = Assets::reserved_balance(MockCurrencyId::USDT, &BOB);
        assert!(not_reserved < reserved && reserved == take_amount);
        assert_ok!(result);
        let order = crate::SellOrders::<Runtime>::get(order_id).unwrap();
        DutchAuction::on_finalize(42);
        let not_found = crate::SellOrders::<Runtime>::get(order_id);
        assert!(not_found.is_none());
	});
}

#[test]
fn flow_with_two_takes_not_enough_for_all() {
	new_test_externalities().execute_with(|| {
		let a = 1_000_000;
		let b = 10;
		Tokens::mint_into(MockCurrencyId::USDT, &BOB, a).unwrap();
		Tokens::mint_into(MockCurrencyId::BTC, &ALICE, b).unwrap();
        let seller = AccountId::from_raw(ALICE.0);
        let buyer = AccountId::from_raw(BOB.0);
        let sell_amount = 3;
        let take_amount = 1000;
        let sell = Sell::new(MockCurrencyId::BTC, MockCurrencyId::USDT, sell_amount, take_amount); 
        let invalid = crate::OrdersIndex::<Runtime>::get();
        let configuration = AuctionStepFunction::LinearDecrease(LinearDecrease {
            total : 42,
         });
        let not_reserved = Assets::reserved_balance(MockCurrencyId::BTC, &ALICE);
        DutchAuction::ask(Origin::signed(seller), sell, configuration).unwrap();
        let reserved = Assets::reserved_balance(MockCurrencyId::BTC, &ALICE);
        assert!(not_reserved < reserved && reserved == sell_amount);
        let order_id = crate::OrdersIndex::<Runtime>::get();
        assert_ne!(invalid, order_id);
        let result = DutchAuction::take(Origin::signed(buyer), order_id, Take::new(1, 999));
        assert!(!result.is_ok());     
        let not_reserved = Assets::reserved_balance(MockCurrencyId::USDT, &BOB);        
        let result = DutchAuction::take(Origin::signed(buyer), order_id, Take::new(1, 1000));
        let reserved = Assets::reserved_balance(MockCurrencyId::USDT, &BOB);
        assert!(not_reserved < reserved && reserved == take_amount);
        assert_ok!(result);
        let order = crate::SellOrders::<Runtime>::get(order_id).unwrap();
        DutchAuction::on_finalize(42);
        let not_found = crate::SellOrders::<Runtime>::get(order_id);
        assert!(not_found.is_none());
	});
}


#[test]
fn liquidation() {
    // ensure order value unreserved
    // ensure that weight for cleanup returned
    // ensure order does not exists
}