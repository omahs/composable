use change_set::{ChangeSet, CheckStorage, Diff, MapChangeSet, OptionChangeSet};
use composable_tests_helpers::test::{block::process_and_progress_blocks, currency::PICA};
use composable_traits::{
	staking::{
		lock::{Lock, LockChangeSet},
		Stake, StakeChangeSet,
	},
	time::ONE_HOUR,
};
use frame_support::traits::ConstU32;
use sp_runtime::Perbill;
use sp_std::collections::btree_map::BTreeMap;

use crate::{
	test::{
		btree_map, create_default_reward_pool, mint_assets, new_test_ext,
		prelude::stake_and_assert,
		runtime::{self, StakingRewards, System, ALICE},
		Test,
	},
	ArgaBlarga, RewardPools, Stakes,
};

#[test]
fn test_diff() {
	let original = Stake {
		reward_pool_id: 1_u32,
		stake: 100_u32,
		share: 120_u32,
		reductions: btree_map::<_, _, ConstU32<10>>([(1_u32, 100_u32), (2, 200), (4, 100)]),
		lock: Lock {
			started_at: 1_000_000,
			duration: 1_000,
			unlock_penalty: Perbill::from_rational(1_u32, 100),
		},
	};

	let new = Stake {
		reward_pool_id: 1,
		stake: 100,
		share: 120,
		reductions: btree_map([(1, 100), (3, 200), (4, 100)]),
		lock: Lock {
			started_at: 1_000_000,
			duration: 2_000,
			unlock_penalty: Perbill::from_rational(1_u32, 100),
		},
	};

	let expected_changes = StakeChangeSet {
		reductions: [
			(1, MapChangeSet::NoChange),
			(2, MapChangeSet::Missing),
			(3, MapChangeSet::Added(200)),
			(4, MapChangeSet::NoChange),
		]
		.into_iter()
		.collect(),
		lock: LockChangeSet { duration: ChangeSet::Changed(2000), ..Default::default() },
		..Default::default()
	};

	dbg!(&expected_changes);

	assert_eq!(expected_changes, original.diff(new));
}

#[test]
fn test_create_reward_pool_diff() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let value_before = Stakes::<Test>::current_value();

		create_default_reward_pool();

		process_and_progress_blocks::<StakingRewards, Test>(1);

		mint_assets([ALICE], [PICA::ID], 100_000_000_000);
		stake_and_assert::<Test, runtime::Event>(ALICE, PICA::ID, 100_000_000, ONE_HOUR);

		assert_eq!(
			Stakes::<Test>::check_storage(value_before),
			BTreeMap::new(),
			// [(
			// 	0,
			// 	MapChangeSet::Added(
			// 		[(
			// 			0,
			// 			(Stake {
			// 				reward_pool_id: 1,
			// 				stake: 123,
			// 				share: 123,
			// 				reductions: BTreeMap::new().try_into().unwrap(),
			// 				lock: Lock {
			// 					started_at: 123,
			// 					duration: 123,
			// 					unlock_penalty: Perbill::from_rational(1_u32, 7)
			// 				}
			// 			})
			// 		)]
			// 		.into_iter()
			// 		.collect()
			// 	)
			// )]
			// .into_iter()
			// .collect()
		);

		// test storage that is set to 100 when creating a pool
		assert_eq!(ArgaBlarga::<Test>::check_storage(Some(100)), OptionChangeSet::NoChange);
		assert_eq!(ArgaBlarga::<Test>::check_storage(Some(10)), OptionChangeSet::Changed(10));

		// assert_eq!(ArgaBlarga::<Test>::check_storage(Some(10)), OptionChangeSet::NoChange);

		// assert_eq!(
		// 	FinancialNft::collections().collect::<BTreeSet<_>>(),
		// 	BTreeSet::from([PICA::ID])
		// );

		// assert_eq!(
		// 	<StakingRewards as FinancialNftProtocol>::collection_asset_ids(),
		// 	vec![PICA::ID]
		// );
	});
}
