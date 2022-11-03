#![feature(generic_associated_types)]

use core::fmt::Debug;

use frame_support::{
	pallet_prelude::{StorageDoubleMap, StorageMap, StorageValue},
	storage::types::QueryKindTrait,
	traits::{Get, StorageInstance},
	BoundedBTreeMap, ReversibleStorageHasher, StorageHasher,
};
use parity_scale_codec::FullCodec;
use sp_arithmetic::fixed_point::FixedU64;
use sp_runtime::{FixedU128, Perbill};
use sp_std::collections::btree_map::BTreeMap;

// TODO: docs lol
pub trait Diffable: Debug {
	// + Default
	type ChangeSet: PartialEq + Debug;

	fn diff(self, updated: Self) -> Diff<Self::ChangeSet>;
}

#[derive(Debug)]
pub enum AssertChanges<T> {
	NoChange,
	IgnoreChange,
	ChangeTo(T),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Diff<T> {
	NoChange,
	Changed(T),
}

impl<T> Default for Diff<T> {
	fn default() -> Self {
		Self::NoChange
	}
}

/// Describes the diff between the values of two map-like structures.
///
/// Not intended to be used as a standalone diff - this should be used with it's associated key in
/// the map.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum MapValueDiff<T: Diffable> {
	/// The item under this key was not changed between the original map and the updated map.
	#[default]
	NoChange,
	/// The item under this key in the original map was not found in the updated map.
	Missing,
	/// The item under this key was not in the original map. Contaianed is the un-diffed value, as
	/// there is nothing to diff it against.
	Added(T),
	/// Th item under this key has changed between the original map and the updated map. Contained
	/// is the diff between the original and updated value.
	Changed(T::ChangeSet),
}

impl<K: Ord + Debug, V: PartialEq + Debug + Diffable, S: Get<u32>> Diffable
	for BoundedBTreeMap<K, V, S>
{
	type ChangeSet = <BTreeMap<K, V> as Diffable>::ChangeSet;

	fn diff(self, updated: Self) -> Diff<Self::ChangeSet> {
		self.into_inner().diff(updated.into_inner())
	}
}

impl<K: Ord + Debug, V: PartialEq + Debug + Diffable> Diffable for BTreeMap<K, V> {
	type ChangeSet = BTreeMap<K, MapValueDiff<V>>;

	fn diff(self, mut updated: Self) -> Diff<Self::ChangeSet> {
		let mut map = self
			.into_iter()
			.map(|(k, v)| match updated.remove(&k) {
				Some(maybe_updated) => match v.diff(maybe_updated) {
					Diff::NoChange => (k, MapValueDiff::NoChange),
					Diff::Changed(changed) => (k, MapValueDiff::Changed(changed)),
				},
				None => (k, MapValueDiff::Missing),
			})
			.collect::<Self::ChangeSet>();

		map.extend(updated.into_iter().map(|(k, v)| (k, MapValueDiff::Added(v))));

		if map.values().all(|v| matches!(v, MapValueDiff::NoChange)) {
			Diff::NoChange
		} else {
			Diff::Changed(map)
		}
	}
}

impl<T: Diffable + PartialEq + Eq + Debug> Diffable for Option<T> {
	type ChangeSet = OptionDiff<T>;

	fn diff(self, updated: Self) -> Diff<Self::ChangeSet> {
		match (self, updated) {
			(None, None) => Diff::NoChange,
			(None, Some(v)) => Diff::Changed(OptionDiff::WasNoneNowSome(v)),
			(Some(_), None) => Diff::Changed(OptionDiff::WasSomeNowNone),
			(Some(old), Some(new)) => match old.diff(new) {
				Diff::NoChange => Diff::NoChange,
				Diff::Changed(changed) => Diff::Changed(OptionDiff::Changed(changed)),
			},
		}
	}
}

/// Describes the diff between two [`Option`]s.
#[derive(Debug, PartialEq, Eq)]
pub enum OptionDiff<T: Diffable> {
	/// The value was previously `Some(x)`, and is now Some(y) where `x != y`.
	Changed(T::ChangeSet),
	/// The value was previously `None` but is now `Some`. Contained is the un-diffed value, as
	/// there is nothing to diff it against.
	WasNoneNowSome(T),
	/// The value was previously `Some`, but is now `None`.
	WasSomeNowNone,
}

macro_rules! impl_diff_primitives {
	($ty: ty) => {
		impl Diffable for $ty {
			type ChangeSet = $ty;

			fn diff(self, updated: Self) -> Diff<Self::ChangeSet> {
				if self == updated {
					Diff::NoChange
				} else {
					Diff::Changed(updated)
				}
			}
		}
	};
}

// unsigned
impl_diff_primitives!(u8);
impl_diff_primitives!(u16);
impl_diff_primitives!(u32);
impl_diff_primitives!(u64);
impl_diff_primitives!(u128);

// signed
impl_diff_primitives!(i8);
impl_diff_primitives!(i16);
impl_diff_primitives!(i32);
impl_diff_primitives!(i64);
impl_diff_primitives!(i128);

// other types that work with this macro
impl_diff_primitives!(Perbill);
impl_diff_primitives!(FixedU128);
impl_diff_primitives!(FixedU64);

// STORAGE

pub trait CheckStorage {
	type Value: Diffable;

	fn current_value() -> Self::Value;

	fn diff_storage_changes(expected: Self::Value) -> Diff<<Self::Value as Diffable>::ChangeSet> {
		expected.diff(Self::current_value())
	}
}

impl<Prefix, Value, QueryKind, OnEmpty> CheckStorage
	for StorageValue<Prefix, Value, QueryKind, OnEmpty>
where
	Prefix: StorageInstance,
	Value: FullCodec + Diffable,
	QueryKind: QueryKindTrait<Value, OnEmpty>,
	QueryKind::Query: Diffable,
	OnEmpty: Get<QueryKind::Query> + 'static,
{
	type Value = QueryKind::Query;

	fn current_value() -> Self::Value {
		Self::get()
	}
}

impl<Prefix, Hasher, Key, Value, QueryKind, OnEmpty, MaxValues> CheckStorage
	for StorageMap<Prefix, Hasher, Key, Value, QueryKind, OnEmpty, MaxValues>
where
	Prefix: StorageInstance,
	Hasher: StorageHasher + ReversibleStorageHasher,
	Key: FullCodec + Debug + Ord,
	Value: FullCodec + PartialEq + Diffable,
	QueryKind: QueryKindTrait<Value, OnEmpty>,
	QueryKind::Query: Diffable,
	OnEmpty: Get<QueryKind::Query> + 'static,
	MaxValues: Get<Option<u32>>,
{
	type Value = BTreeMap<Key, Value>;

	fn current_value() -> Self::Value {
		Self::iter().collect::<BTreeMap<_, _>>()
	}
}

impl<Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind, OnEmpty, MaxValues> CheckStorage
	for StorageDoubleMap<Prefix, Hasher1, Key1, Hasher2, Key2, Value, QueryKind, OnEmpty, MaxValues>
where
	Prefix: StorageInstance,
	Hasher1: StorageHasher + ReversibleStorageHasher,
	Key1: FullCodec + Debug + Ord,
	Hasher2: StorageHasher + ReversibleStorageHasher,
	Key2: FullCodec + Debug + Ord,
	Value: FullCodec + PartialEq + Diffable,
	QueryKind: QueryKindTrait<Value, OnEmpty>,
	OnEmpty: Get<QueryKind::Query> + 'static,
	MaxValues: Get<Option<u32>>,
{
	type Value = BTreeMap<Key1, BTreeMap<Key2, Value>>;

	fn current_value() -> Self::Value {
		let mut found_map = BTreeMap::new();

		for (k1, k2, v) in Self::iter() {
			dbg!(&k1, &k2);
			found_map.entry(k1).or_insert_with(BTreeMap::<Key2, Value>::new).insert(k2, v);
		}

		found_map
	}
}
