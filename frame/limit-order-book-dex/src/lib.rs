//! on chain state to handle state of cross chain exchanges
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
	bad_style,
	bare_trait_objects,
	const_err,
	improper_ctypes,
	non_shorthand_field_patterns,
	no_mangle_generic_items,
	overflowing_literals,
	path_statements,
	patterns_in_fns_without_body,
	private_in_public,
	unconditional_recursion,
	unused_allocation,
	unused_comparisons,
	unused_parens,
	while_true,
	trivial_casts,
	trivial_numeric_casts
)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod mocks;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Codec, Decode, Encode, FullCodec};
	use composable_traits::{
		auction::{AuctionState, AuctionStepFunction, DutchAuction},
		dex::{AmmExchange, LimitOrderbook, LimitOrderbook, Price},
		loans::{DurationSeconds, Timestamp, ONE_HOUR},
		math::{LiftedFixedBalance, SafeArithmetic, WrappingNext},
	};
	use frame_support::{
		ensure,
		pallet_prelude::{MaybeSerializeDeserialize, ValueQuery},
		traits::{
			fungibles::{Inspect, Mutate, Transfer},
			tokens::WithdrawConsequence,
			Currency, IsType, UnixTime,
		},
		Parameter, StorageMap, Twox64Concat,
	};
	use scale_info::TypeInfo;

	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, Account};
	use num_traits::{CheckedDiv, SaturatingAdd, SaturatingSub, WrappingAdd};

	use sp_runtime::{
		traits::{
			AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub, One,
			Saturating, Zero,
		},
		ArithmeticError, DispatchError, FixedPointNumber, FixedPointOperand, FixedU128, Percent,
		Permill, Perquintill,
	};
	use sp_std::{fmt::Debug, vec::Vec};

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: DeFiComposableConfig {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type UnixTime: UnixTime;
		type Orderbook: LimitOrderbook<
			AssetId = Self::AssetId,
			Balance = Self::Balance,
			AccountId = Self::AccountId,
			OrderId = Self::DexOrderId,
		>;
		type DexOrderId: FullCodec + Default + TypeInfo;
		type OrderId: FullCodec + Clone + Debug + Eq + Default + WrappingNext + TypeInfo;		
	}

	// type aliases
	pub type OrderIdOf<T> = <T as Config>::Orderbook::OrderId;
	pub type OrderOf<T> = Order<OrderIdOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	/// auction can span several dex orders within its lifetime
	#[derive(Encode, Decode, Default, TypeInfo)]
	pub struct Order<OrderId, AssetId> {
		pub id: OrderId,
		pub asset_id: AssetId,
	}

	// /// All registered buys
	// #[pallet::storage]
	// #[pallet::getter(fn buys)]
	// pub type Buys<T: Config> = StorageMap<_, Twox64Concat, T::OrderId, OrderOf<T>, ValueQuery>;

	// #[pallet::storage]
	// #[pallet::getter(fn orders_index)]
	// pub type OrdersIndex<T: Config> = StorageValue<_, T::OrderId, ValueQuery>;

	// impl<T: Config + DeFiComposableConfig> LimitOrderbook for Pallet<T> {
	// 	type OrderId = T::OrderId;

	// 	type AmmDex = ();

	// 	type AmmConfiguration;

	// 	fn ask(
	// 		from: &Self::AccountId,
	// 		to: &Self::AccountId,
	// 		order: composable_traits::dex::Sell<Self::AssetId, Self::Balance>,
	// 		in_amount: Self::Balance,
	// 		amm: Self::AmmConfiguration,
	// 	) -> Result<Self::OrderId, DispatchError> {
	// 		todo!()
	// 	}

	// 	fn bid(
	// 		account_from: &Self::AccountId,
	// 		to: &Self::AccountId,
	// 		order: composable_traits::dex::Buy<Self::AssetId, Self::Balance>,
	// 		in_amount: Self::Balance,
	// 		amm: Self::AmmConfiguration,
	// 	) -> Result<Self::OrderId, DispatchError> {
	// 		todo!()
	// 	}

	// 	fn patch(order_id: Self::OrderId, price: Self::Balance) -> Result<(), DispatchError> {
	// 		todo!()
	// 	}

	// 	fn take(
	// 		from: &Self::AccountId,
	// 		to: &Self::AccountId,
	// 		order: Self::OrderId,
	// 		amount: Self::Balance,
	// 		limit: Self::Balance,
	// 	) -> Result<(), DispatchError> {
	// 		todo!()
	// 	}
	//}
}
