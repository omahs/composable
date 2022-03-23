#[allow(unused_imports)]
use crate::mock::{Event, ExtBuilder, MockRuntime, Options, Origin, System, ALICE, BOB};
use crate::*;
use crate::{pallet, pallet::Error};
use frame_support::assert_ok;

#[test]
fn call_test_extrinsic() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Options::test(Origin::signed(ALICE)));

		let event = <frame_system::Pallet<MockRuntime>>::events()
			.pop()
			.expect("Expected at least one EventRecord to be found")
			.event;

		// System::assert_last_event(Event::Options(pallet::Event::Test { issuer: ALICE }));
		assert_eq!(event, Event::Options(pallet::Event::Test { issuer: ALICE }))
	});
}
