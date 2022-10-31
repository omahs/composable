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
	type ChangeSet: PartialEq + Debug + Default;

	fn diff(self, updated: Self) -> Self::ChangeSet;
}

#[derive(Debug)]
pub enum AssertChanges<T> {
	NoChange,
	IgnoreChange,
	ChangeTo(T),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Diff<T: Diffable> {
	#[default]
	NoChange,
	Changed(T::ChangeSet),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum MapDiff<T: Diffable> {
	#[default]
	NoChange,
	Missing,
	Added(T),
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
	type ChangeSet = BTreeMap<K, MapDiff<V>>;

	fn diff(self, mut updated: Self) -> Self::ChangeSet {
		let mut map = self
			.into_iter()
			.map(|(k, v)| match updated.remove(&k) {
				Some(maybe_updated) =>
					if maybe_updated == v {
						(k, MapDiff::NoChange)
					} else {
						(k, MapDiff::Changed(v.diff(maybe_updated)))
					},
				None => (k, MapDiff::Missing),
			})
			.collect::<Self::ChangeSet>();

		map.extend(updated.into_iter().map(|(k, v)| (k, MapDiff::Added(v))));

		map
	}
}

impl<T: Diffable + PartialEq + Eq + Debug> Diffable for Option<T> {
	type ChangeSet = OptionDiff<T>;

	fn diff(self, updated: Self) -> Self::ChangeSet {
		match (self, updated) {
			(None, None) => OptionDiff::NoChange,
			(None, Some(v)) => OptionDiff::WasNoneNowSome(v),
			(Some(_), None) => OptionDiff::WasSomeNowNone,
			(Some(old), Some(new)) =>
				if new == old {
					OptionDiff::NoChange
				} else {
					OptionDiff::Changed(new)
				},
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum OptionDiff<T: Diffable> {
	NoChange,
	Changed(T::ChangeSet),
	WasNoneNowSome(T),
	WasSomeNowNone,
}

impl<T> Default for OptionDiff<T> {
	fn default() -> Self {
		Self::NoChange
	}
}

macro_rules! impl_diff_with_implicit_eq {
	($ty: ty, $ChangeSet: ty) => {
		impl DiffTrait for $ty {
			type ChangeSet<T: DiffTrait> = ChangeSet;

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
impl_diff_with_implicit_eq!(u8);
impl_diff_with_implicit_eq!(u16);
impl_diff_with_implicit_eq!(u32);
impl_diff_with_implicit_eq!(u64);
impl_diff_with_implicit_eq!(u128);

// signed
impl_diff_with_implicit_eq!(i8);
impl_diff_with_implicit_eq!(i16);
impl_diff_with_implicit_eq!(i32);
impl_diff_with_implicit_eq!(i64);
impl_diff_with_implicit_eq!(i128);

// other types that work with this macro
impl_diff_with_implicit_eq!(Perbill);
impl_diff_with_implicit_eq!(FixedU128);
impl_diff_with_implicit_eq!(FixedU64);

// STORAGE

pub trait CheckStorage {
	type Value: Diffable;
	type Diff<T: Diffable>;

	fn current_value() -> Self::Value;

	fn check_storage(expected: Self::Value) -> Self::Diff<Self::Value> {
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
			found_map
				.entry(k1)
				.and_modify(|e: &mut BTreeMap<Key2, Value>| {
					e.insert(k2, v);
				})
				.or_insert_with(BTreeMap::<Key2, Value>::new);
		}

		found_map
	}
}
