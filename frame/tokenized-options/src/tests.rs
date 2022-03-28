use crate::currency::BTC;
use crate::currency::PICA;

use crate::mock::{Event, ExtBuilder, MockRuntime, Origin, System, TokenizedOptions, ALICE, BOB};
use crate::{pallet, pallet::Error};

use frame_support::assert_ok;

#[test]
fn call_test_extrinsic() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TokenizedOptions::create(Origin::signed(ALICE), PICA::ID));

		let event = <frame_system::Pallet<MockRuntime>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// System::assert_last_event(Event::Options(pallet::Event::Test { issuer: ALICE }));
		assert_eq!(event, Event::TokenizedOptions(pallet::Event::Create { asset: PICA::ID }));
		// assert_ne!(event, Event::TokenizedOptions(pallet::Event::Test { issuer: BOB }))
	});
}

#[test]
fn call_create_extrinsic() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(TokenizedOptions::create(Origin::signed(ALICE), PICA::ID));
	});
}

#[test]
fn test_create_emits_event() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(TokenizedOptions::create(Origin::signed(ALICE), PICA::ID));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::Create {
			asset: PICA::ID,
		}));

		assert_ok!(TokenizedOptions::create(Origin::signed(BOB), BTC::ID));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::Create {
			asset: BTC::ID,
		}));
	});
}
