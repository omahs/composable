use core::fmt::Debug;

use composable_traits::{
	staking::{lock::Lock, Stake},
	time::{DurationSeconds, Timestamp},
};
use frame_support::{
	traits::{ConstU32, Get},
	BoundedBTreeMap, DebugNoBound,
};
use sp_runtime::Perbill;
use sp_std::collections::btree_map::BTreeMap;

use super::btree_map;

#[derive(Debug)]
pub enum AssertChanges<T> {
	NoChange,
	IgnoreChange,
	ChangeTo(T),
}

#[derive(Debug, Default)]
pub enum ChangeSet<T> {
	#[default]
	NoChange,
	Changed(T),
}

pub trait Diff {
	type ChangeSet: Debug + Default;

	fn diff(self, updated: Self) -> Self::ChangeSet;
}

#[derive(Debug, Default)]
pub enum MapChangeSet<T> {
	Missing,
	Added(T),
	#[default]
	NoChange,
	Changed(T),
}

impl<K: Ord + Debug, V: PartialEq + Debug, S: Get<u32>> Diff for BoundedBTreeMap<K, V, S> {
	type ChangeSet = BTreeMap<K, MapChangeSet<V>>;

	fn diff(self, mut updated: Self) -> Self::ChangeSet {
		let mut map = self
			.into_iter()
			.map(|(k, v)| match updated.remove(&k) {
				Some(maybe_updated) =>
					if maybe_updated == v {
						(k, MapChangeSet::NoChange)
					} else {
						(k, MapChangeSet::Changed(maybe_updated))
					},
				None => (k, MapChangeSet::Missing),
			})
			.collect::<Self::ChangeSet>();

		map.extend(updated.into_iter().map(|(k, v)| (k, MapChangeSet::Added(v))));

		map
	}
}

#[derive(DebugNoBound, Default)]
pub struct StakeChangeSet<
	AssetId: Diff + Ord + Debug,
	RewardPoolId: Diff + Debug,
	Balance: Diff + PartialEq + Debug,
	MaxReductions: Get<u32>,
> {
	/// Reward Pool ID from which pool to allocate rewards for this
	pub reward_pool_id: RewardPoolId::ChangeSet,

	/// The original stake this position was created for or updated position with any extended
	/// stake amount.
	pub stake: Balance::ChangeSet,

	/// Pool share received for this position
	pub share: Balance::ChangeSet,

	/// Reduced rewards by asset for the position (d_n)
	pub reductions: <BoundedBTreeMap<AssetId, Balance, MaxReductions> as Diff>::ChangeSet,

	/// The lock period for the stake.
	pub lock: <Lock as Diff>::ChangeSet,
}

#[derive(Debug, Default)]
pub struct LockChangeSet {
	/// The date at which this NFT was minted or to which lock was extended too.
	pub started_at: <Timestamp as Diff>::ChangeSet,
	/// The duration for which this NFT stake was locked.
	pub duration: <DurationSeconds as Diff>::ChangeSet,

	pub unlock_penalty: <Perbill as Diff>::ChangeSet,
}

impl Diff for Lock {
	type ChangeSet = LockChangeSet;

	fn diff(self, updated: Self) -> Self::ChangeSet {
		LockChangeSet {
			started_at: self.started_at.diff(updated.started_at),
			duration: self.duration.diff(updated.duration),
			unlock_penalty: self.unlock_penalty.diff(updated.unlock_penalty),
		}
	}
}

impl<AssetId, RewardPoolId, Balance, MaxReductions> Diff
	for Stake<AssetId, RewardPoolId, Balance, MaxReductions>
where
	AssetId: Diff + Ord + Debug + PartialEq + Eq + Clone,
	RewardPoolId: Diff + Debug + PartialEq + Eq + Clone,
	Balance: Diff + Debug + PartialEq + Eq + Clone,
	MaxReductions: Get<u32>,
{
	type ChangeSet = StakeChangeSet<AssetId, RewardPoolId, Balance, MaxReductions>;

	fn diff(self, updated: Self) -> Self::ChangeSet {
		StakeChangeSet {
			reward_pool_id: self.reward_pool_id.diff(updated.reward_pool_id),
			stake: self.stake.diff(updated.stake),
			share: self.share.diff(updated.share),
			reductions: self.reductions.diff(updated.reductions),
			lock: self.lock.diff(updated.lock),
		}
	}
}

macro_rules! impl_diff {
	($ty: ty) => {
		impl Diff for $ty {
			type ChangeSet = ChangeSet<$ty>;

			fn diff(self, updated: Self) -> Self::ChangeSet {
				if self == updated {
					ChangeSet::NoChange
				} else {
					ChangeSet::Changed(updated)
				}
			}
		}
	};
}

impl_diff!(u8);
impl_diff!(u16);
impl_diff!(u32);
impl_diff!(u64);
impl_diff!(u128);

impl_diff!(i8);
impl_diff!(i16);
impl_diff!(i32);
impl_diff!(i64);
impl_diff!(i128);

impl_diff!(Perbill);

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

	// assert_eq!(expected_changes, original.diff(new));
}
