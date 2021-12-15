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
		dex::{AmmExchange, LimitOrderbook, Price, Buy, Sell, Take},
		loans::{DurationSeconds, Timestamp, ONE_HOUR},
		math::{LiftedFixedBalance, SafeArithmetic, WrappingNext}, defi::DeFiComposableConfig,
	};
	use frame_support::{
		ensure,
		pallet_prelude::*,
		traits::{
			fungibles::{Inspect, Mutate, Transfer},
			tokens::WithdrawConsequence,
			Currency, IsType, UnixTime,
		},		
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
	pub trait Config: DeFiComposableConfig + frame_system::Config  {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type UnixTime: UnixTime;
		type Orderbook: LimitOrderbook<
			AssetId = Self::AssetId,
			Balance = Self::Balance,
			AccountId = <Self as frame_system::Config>::AccountId,
			OrderId = Self::DexOrderId,
		>;
		type DexOrderId: FullCodec + Copy + Eq + PartialEq + TypeInfo;
		type OrderId: FullCodec + Clone + Debug + Eq + Default + WrappingNext + TypeInfo + sp_std::hash::Hash + WrappingAdd;		
	}

	// type aliases
	pub type OrderIdOf<T> = <<T as Config>::Orderbook as LimitOrderbook>::OrderId;
	pub type BuyOrderOf<T> = BuyOrder<OrderIdOf<T>, <T as DeFiComposableConfig>::AssetId, <T as DeFiComposableConfig>::Balance>;
	pub type SellOrderOf<T> = SellOrder<OrderIdOf<T>, <T as DeFiComposableConfig>::AssetId, <T as DeFiComposableConfig>::Balance>;

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

	#[derive(Encode, Decode, Default, TypeInfo)]
	pub struct TakeBy<AccountId, Balance> {
		pub from_to: AccountId,
		pub take: Take<Balance>,
	} 

	#[derive(Encode, Decode, Default, TypeInfo)]
	pub struct BuyOrder<OrderId, AssetId, Balance> {
		pub id: OrderId,
		pub order: Buy<AssetId, Balance>,
		pub takes: Vec<TakeBy>,
	}

	#[derive(Encode, Decode, Default, TypeInfo)]
	pub struct SellOrder<OrderId, AssetId, Balance> {
		pub id: OrderId,
		pub order: Sell<AssetId, Balance>,
		pub takes: Vec<TakeBy>,
	}

	#[pallet::storage]
	#[pallet::getter(fn buys)]
	pub type BuyOrders<T: Config> = StorageMap<
		_,
		Twox64Concat,
		OrderIdOf<T>,
		BuyOrderOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn buys)]
	pub type SellOrders<T: Config> = StorageMap<
		_,
		Twox64Concat,
		OrderIdOf<T>,
		SellOrderOf<T>,
		OptionQuery,
	>;

	
	#[pallet::storage]
	#[pallet::getter(fn orders_index)]
	pub type OrdersIndex<T: Config> = StorageValue<_, T::OrderId, ValueQuery>;

	impl<T: Config + DeFiComposableConfig> LimitOrderbook for Pallet<T> {
    type OrderId ;

    type AmmDex ;

    type AmmConfiguration ;

    fn ask(
		from_to: &Self::AccountId,
		order: Sell<Self::AssetId, Self::Balance>,		
		base_amount: Self::Balance,
		amm: Self::AmmConfiguration,
	) -> Result<Self::OrderId, DispatchError> {
        let order_id = OrdersIndex::<T>::try_mutate(|x| {
			x.wrapping_add()
		});		

		let order = SellOrderOf {
			id: order_id,
			order: order,
			takes: <_>::default(),
		};

		// we not actually care if there was order before under same index, because `take` accepts risk limiter parameter
		SellOrder::<T>::insert(order_id, order);		
		Ok(())
    }

    fn bid(
		from_to: &Self::AccountId,		
		order: Buy<Self::AssetId, Self::Balance>,		
		base_amount: Self::Balance,
		amm: Self::AmmConfiguration,
	) -> Result<Self::OrderId, DispatchError> {
        todo!()
    }

    fn patch(
		order_id: Self::OrderId,
		price: Self::Balance,
	) -> Result<(), DispatchError> {
        todo!()
    }

    fn take(
		from_to: &Self::AccountId,
		order: Self::OrderId,
		take : Take<Self::Balance>,
	) -> Result<(), DispatchError> {
		// here we add take on order, will live only one block, not stored on chain
		// 
        BlockOrders::<T>::upsert(order_id, take)
    }
}
}
