use core::fmt::Debug;
use std::{convert::Infallible, marker::PhantomData};

use frame_support::{
	pallet_prelude::{StorageDoubleMap, StorageMap, StorageValue},
	storage::types::QueryKindTrait,
	traits::{Get, StorageInstance},
	BoundedBTreeMap, ReversibleStorageHasher, StorageHasher,
};
// use frunk::hlist;
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

// #[derive(Debug)]
// pub enum AssertChanges<T> {
// 	NoChange,
// 	IgnoreChange,
// 	ChangeTo(T),
// }

#[derive(Debug, PartialEq, Eq)]
pub enum Diff<T> {
	NoChange,
	Changed(T),
}

// impl<T> From<T> for Diff<T> {
// 	fn from(t: T) -> Self {
// 		Diff::Changed(t)
// 	}
// }

pub enum DiffComparisonResult<T> {
	Same,
	UnexpectedChange(T),
	ExpectedChange(T),
	ChangeIsSame,
	ChangeIsNotSame(T, T),
}

impl<T: PartialEq> Diff<T> {
	pub fn compare(self, other: Self) -> DiffComparisonResult<T> {
		match (self, other) {
			(Diff::NoChange, Diff::NoChange) => DiffComparisonResult::Same,
			(Diff::NoChange, Diff::Changed(t)) => DiffComparisonResult::ExpectedChange(t),
			(Diff::Changed(t), Diff::NoChange) => DiffComparisonResult::UnexpectedChange(t),
			(Diff::Changed(original), Diff::Changed(expected)) =>
				if original == expected {
					DiffComparisonResult::Same
				} else {
					DiffComparisonResult::ChangeIsNotSame(original, expected)
				},
		}
	}
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

impl Diffable for () {
	type ChangeSet = Infallible;

	fn diff(self, _: Self) -> Diff<Self::ChangeSet> {
		Diff::NoChange
	}
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

	fn diff_storage_changes_with_expected_changes(
		expected: Self::Value,
	) -> Diff<<Self::Value as Diffable>::ChangeSet> {
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

// name is bikeshedding lol
pub struct AssertableDiffableStorageAction<
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
	F: FnOnce(),
> {
	f: F,
	storage_checker: StorageChecker<UncheckedStorages, CheckedStorages>,
	// _marker: PhantomData<fn() -> S>,
}

pub fn do_action<UncheckedStorages: PalletStorageHList, F: FnOnce()>(
	f: F,
) -> AssertableDiffableStorageAction<UncheckedStorages, (), F> {
	AssertableDiffableStorageAction {
		f,
		storage_checker: StorageChecker { expected_changes: (), _marker: PhantomData },
	}
}

impl<CheckedStorages, UncheckedStorages, F>
	AssertableDiffableStorageAction<UncheckedStorages, CheckedStorages, F>
where
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
	UncheckedStorages::Output: Concat<CheckedStorages::Output>,
	F: FnOnce(),
{
	#[must_use = "check_storage does nothing on it's own, assert_storage_changes must be called to actually do the checks"]
	pub fn check_storage<T: CheckStorage, Index>(
		self,
		t_value: T::Value,
	) -> AssertableDiffableStorageAction<
		<UncheckedStorages as Find<T, Index>>::Remainder,
		(T, CheckedStorages),
		F,
	>
	where
		UncheckedStorages: Find<T, Index>,
		<UncheckedStorages as Find<T, Index>>::Remainder: PalletStorageHList,
	{
		AssertableDiffableStorageAction {
			f: self.f,
			storage_checker: self.storage_checker.check_storage(t_value),
		}
	}

	pub fn assert_storage_changes(
		self,
		// expected_changes: Diff<<S::Value as Diffable>::ChangeSet>,
		// expected_changes: CheckedStorages::Input,
		// ) -> DiffComparisonResult<<S::Value as Diffable>::ChangeSet> {
	) -> <UncheckedStorages::Output as Concat<CheckedStorages::Output>>::Output {
		// let cv = CheckedStorages::current_value();

		// (self.f)();

		self.storage_checker.check(self.f)

		// check_res.compare(expected_changes)
	}
}

/// Generic HList trait
pub trait HList {}

impl HList for () {}

impl<Head, Tail> HList for (Head, Tail) where Tail: HList {}

/// Concat two HLists
pub trait Concat<Rhs> {
	type Output;

	fn concat(self, rhs: Rhs) -> Self::Output;
}

impl<Rhs> Concat<Rhs> for ()
where
	Rhs: HList,
{
	type Output = Rhs;

	fn concat(self, rhs: Rhs) -> Rhs {
		rhs
	}
}

impl<Head, Tail, Rhs> Concat<Rhs> for (Head, Tail)
where
	Tail: Concat<Rhs>,
	Rhs: HList,
{
	type Output = (Head, <Tail as Concat<Rhs>>::Output);

	fn concat(self, rhs: Rhs) -> Self::Output {
		(self.0, self.1.concat(rhs))
	}
}

/// HList trait specific to the pallet storages
pub trait PalletStorageHList: HList {
	type Input;
	type Output;

	fn current_value() -> Self::Input;

	fn diff_storage_changes_with_expected_changes(expected: Self::Input) -> Self::Output;
}

impl PalletStorageHList for () {
	type Input = ();
	type Output = ();

	fn current_value() -> Self::Input {}

	fn diff_storage_changes_with_expected_changes(_: Self::Input) -> Self::Output {}
}

impl<Head, Tail> PalletStorageHList for (Head, Tail)
where
	Head: CheckStorage,
	Tail: PalletStorageHList,
{
	type Input = (Head::Value, Tail::Input);
	type Output = (Diff<<Head::Value as Diffable>::ChangeSet>, Tail::Output);

	fn current_value() -> Self::Input {
		(Head::current_value(), Tail::current_value())
	}

	fn diff_storage_changes_with_expected_changes(expected: Self::Input) -> Self::Output {
		(
			Head::diff_storage_changes_with_expected_changes(expected.0),
			Tail::diff_storage_changes_with_expected_changes(expected.1),
		)
	}
}

/// "Builder" for storage checks
pub struct StorageChecker<UncheckedStorages, CheckedStorages>
where
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
{
	expected_changes: CheckedStorages::Input,
	_marker: PhantomData<fn() -> UncheckedStorages>,
}

impl<UncheckedStorages: PalletStorageHList, CheckedStorages: PalletStorageHList>
	StorageChecker<UncheckedStorages, CheckedStorages>
{
	/// Adds a check for the storage `T`, moving it from `UncheckedStorages` to `CheckedStorages` in
	/// doing so.
	pub fn check_storage<T: CheckStorage, Index>(
		self,
		t_value: T::Value,
	) -> StorageChecker<<UncheckedStorages as Find<T, Index>>::Remainder, (T, CheckedStorages)>
	where
		UncheckedStorages: Find<T, Index>,
		<UncheckedStorages as Find<T, Index>>::Remainder: PalletStorageHList,
	{
		StorageChecker { expected_changes: (t_value, self.expected_changes), _marker: PhantomData }
	}
}

impl<PalletStorages: PalletStorageHList> StorageChecker<PalletStorages, ()> {
	/// Creates a new [`StorageTester`] with the provided pallet storages and all storages
	/// unchecked.
	#[allow(clippy::new_without_default)]
	pub fn new() -> StorageChecker<PalletStorages, ()> {
		StorageChecker { expected_changes: (), _marker: PhantomData }
	}
}

impl<UncheckedStorages, CheckedStorages> StorageChecker<UncheckedStorages, CheckedStorages>
where
	UncheckedStorages: PalletStorageHList,
	CheckedStorages: PalletStorageHList,
	UncheckedStorages::Output: Concat<CheckedStorages::Output>,
{
	pub fn check<F: FnOnce()>(
		self,
		f: F,
	) -> <UncheckedStorages::Output as Concat<CheckedStorages::Output>>::Output {
		let unchecked_value_before_f = UncheckedStorages::current_value();
		let checked_value_before_f = CheckedStorages::current_value();

		f();

		// this should be equal to self.input
		let checked_diff =
			CheckedStorages::diff_storage_changes_with_expected_changes(checked_value_before_f);

		// this should result in no changes, assuming the storages haven't been changed. if there
		// have been unaccounted for changes, then this will result in a failed diff
		let unchecked_diff =
			UncheckedStorages::diff_storage_changes_with_expected_changes(unchecked_value_before_f);

		unchecked_diff.concat(checked_diff)
	}
}

// fn desired_api() {
// 	StorageTester::new()
// 		.do_action(|| {})
// 		.assert_storage::<_>()
// 		.assert_storage::<_>()
// 		.assert_storage::<_>();
// }

/// Used as an index into an `HList`.
///
/// `Here` is 0, pointing to the head of the HList.
///
/// Users should normally allow type inference to create this type
pub enum Here {}

/// Used as an index into an `HList`.
///
/// `There<T>` is 1 + `T`.
///
/// Users should normally allow type inference to create this type.
pub struct There<T>(std::marker::PhantomData<T>);

// similar to frunk::Selector
pub trait Find<T, I> {
	type Remainder;
}

impl<T, Tail> Find<T, Here> for (T, Tail) {
	type Remainder = Tail;
}

impl<Head, T, Tail, TailIndex> Find<T, There<TailIndex>> for (Head, Tail)
where
	Tail: Find<T, TailIndex>,
{
	type Remainder = (Head, <Tail as Find<T, TailIndex>>::Remainder);
}

struct One;
struct Two;
struct Three;

impl CheckStorage for One {
	type Value = u8;

	fn current_value() -> Self::Value {
		1
	}
}

impl CheckStorage for Two {
	type Value = u16;

	fn current_value() -> Self::Value {
		2
	}
}

impl CheckStorage for Three {
	type Value = u32;

	fn current_value() -> Self::Value {
		3
	}
}

type HListT = (One, (Two, (Three, ())));
// type HListT = (One, (Two, ()));

#[test]
fn abc() {
	let res = do_action::<HListT, _>(|| {})
		.check_storage::<Two, _>(3)
		.check_storage::<Three, _>(1)
		.assert_storage_changes();

	dbg!(res);
	// .check(|| {});
	// .check();
	// let _: <HList as Find<u16, _>>::Remainder = 1;
	// let _: <HList as Find<_, There<There<There<Here>>>>>::Type = 1_u64;
	// let _: <HList as Find<u32, _>>::Index = There::<Here>(PhantomData);
}

// start with hlist of pallet storages:
// struct StorageChecker<CheckedStorages, UncheckedStorages>;
// starts off as StorageChecker<(), PalletStoragesHList>;
//
// on each storage check:
// Find<Storage<T>, _>::Remainder becomes the new storage type, add Storage<T> to the checked
//
// maybe:
// storages StorageChecker::build() wraps the CheckedStorages in Check<Storage<T>>, and the
// UncheckedStorages in AssumeNoChanges<Storage<T>>
