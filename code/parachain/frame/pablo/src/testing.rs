use core::fmt::Debug;

use composable_tests_helpers::{
	test::{currency::PICA, helper::RuntimeTrait},
	ALICE, BOB,
};
use frame_support::{
	pallet_prelude::Member,
	traits::{fungibles::Mutate, OriginTrait, TryCollect},
	Parameter,
};
use frame_system::pallet_prelude::OriginFor;
use sp_arithmetic::Permill;
use sp_runtime::{
	traits::{IdentifyAccount, Verify, Zero},
	MultiSignature,
};

use crate::{Config, Event, Pallet, PoolInitConfiguration};

pub fn new_test_ext_generic<Runtime>() -> sp_io::TestExternalities
where
	Runtime: Config + frame_system::Config,
{
	frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.expect("success")
		.into()
}

pub fn add_liquidity<Runtime>()
where
	Runtime: frame_system::Config<
		AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId,
		BlockNumber = u32,
	>,
	Runtime: Config,
	<Runtime as Config>::AssetId: From<u128>,
	<Runtime as Config>::Balance: From<u128>,
	<Runtime as frame_system::Config>::Event:
		Parameter + Member + Debug + Clone + TryInto<Event<Runtime>> + From<Event<Runtime>>,
	<<Runtime as frame_system::Config>::Event as TryInto<Event<Runtime>>>::Error: Debug,
	<Runtime as frame_system::Config>::Origin:
		OriginTrait<AccountId = <Runtime as frame_system::Config>::AccountId>,
	Runtime: RuntimeTrait<Event<Runtime>> + Config, /* crate::Event<Runtime>: Clone +
	                                                 * Debug + PartialEq, */
{
	// for keep_alive in [true, false] {
	new_test_ext_generic::<Runtime>().execute_with(|| {
		// next_block::<crate::Pallet<Runtime>, Runtime>();

		// frame_system::Pallet::<Runtime>::initialize();
		frame_system::Pallet::<Runtime>::set_block_number(1_u32);

		let asset_1_id: <Runtime as Config>::AssetId = 1_u128.into();
		let asset_2_id: <Runtime as Config>::AssetId = 131_u128.into();

		let pool_id = Runtime::assert_extrinsic_event_with(
			Pallet::<Runtime>::create(
				OriginFor::<Runtime>::root(),
				PoolInitConfiguration::DualAssetConstantProduct {
					owner: ALICE,
					assets_weights: [
						(asset_1_id, Permill::from_parts(500_000)),
						(asset_2_id, Permill::from_parts(500_000)),
					]
					.into_iter()
					.try_collect()
					.unwrap(),
					fee: Permill::from_parts(10_000),
				},
			),
			|event| match event {
				Event::PoolCreated { pool_id, .. } => Some(pool_id),
				_ => None,
			},
		);

		<Runtime as Config>::Assets::mint_into(asset_1_id, &BOB, PICA::units(1_100_000).into())
			.unwrap();
		<Runtime as Config>::Assets::mint_into(asset_2_id, &BOB, PICA::units(1_100_000).into())
			.unwrap();

		let assets = [
			(asset_1_id, PICA::units(1_000_000).into()),
			(asset_2_id, PICA::units(1_000_000).into()),
		]
		.into_iter()
		.collect();

		Runtime::assert_extrinsic_event(
			Pallet::<Runtime>::add_liquidity(
				OriginFor::<Runtime>::signed(BOB),
				pool_id,
				assets,
				Zero::zero(),
				true,
			),
			Event::<Runtime>::LiquidityAdded {
				who: BOB,
				pool_id,
				minted_lp: 1_999_999_994_552_971_605.into(),
			},
		);
	});
	// }
}
