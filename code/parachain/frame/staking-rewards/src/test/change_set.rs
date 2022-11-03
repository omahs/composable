use change_set::{CheckStorage, Diff, Diffable, MapValueDiff, OptionDiff};
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
		reductions: Diff::Changed(
			[
				(1, MapValueDiff::NoChange),
				(2, MapValueDiff::Missing),
				(3, MapValueDiff::Added(200)),
				(4, MapValueDiff::NoChange),
			]
			.into_iter()
			.collect(),
		),
		lock: Diff::Changed(LockChangeSet { duration: Diff::Changed(2000), ..Default::default() }),
		..Default::default()
	};

	dbg!(&expected_changes);

	assert_eq!(Diff::Changed(expected_changes), original.diff(new));
}

// #[macro_export]
macro_rules! btree_map {
  {$($k: expr => $v: expr),* $(,)?} => {
    {
      let mut map = ::sp_std::collections::btree_map::BTreeMap::new();

      $(
        map.insert($k, $v);
      )*

      map
    }
  };
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
			Stakes::<Test>::diff_storage_changes(value_before),
			Diff::Changed(btree_map! {
				1 => MapValueDiff::Added(btree_map! {
					0 => Stake {
						reward_pool_id: 1,
						stake: 100_000_000,
						share: 101_000_000,
						reductions: btree_map! {
							1000 => 0
						}.try_into().unwrap(),
						lock: Lock {
							started_at: 12,
							duration: 3600,
							unlock_penalty: Perbill::from_rational::<u128>(5, 100)
						}
					}
				})
			})
		);

		// test storage that is set to 100 when creating a pool
		assert_eq!(ArgaBlarga::<Test>::diff_storage_changes(Some(100)), Diff::NoChange);
		assert_eq!(
			ArgaBlarga::<Test>::diff_storage_changes(Some(10)),
			Diff::Changed(OptionDiff::Changed(10))
		);

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
