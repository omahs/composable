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
	trivial_numeric_casts,
	unused_extern_crates
)]

#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub use pallet::*;
mod math;

#[frame_support::pallet]
pub mod pallet {

	use codec::{Codec, Decode, Encode, FullCodec};
	use composable_traits::{
		auction::{AuctionState, AuctionStepFunction, DutchAuction},
		loans::{DurationSeconds, Timestamp, ONE_HOUR},
		math::{LiftedFixedBalance, SafeArithmetic, WrappingNext}, defi::{DeFiComposableConfig, OrderIdLike},
	};
	use frame_support::{
		ensure,
		pallet_prelude::{MaybeSerializeDeserialize, ValueQuery},
		traits::{
			fungibles::{Inspect, Mutate, Transfer},
			tokens::WithdrawConsequence,
			Currency, IsType, UnixTime,
		},
		Parameter, Twox64Concat,
	};

	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, Account};
	use num_traits::{CheckedDiv, SaturatingAdd, SaturatingSub, WrappingAdd};

	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{
			AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub, One,
			Saturating, Zero,
		},
		ArithmeticError, DispatchError, FixedPointNumber, FixedPointOperand, FixedU128, Percent,
		Permill, Perquintill,
	};
	use sp_std::{fmt::Debug, vec::Vec};

	use crate::math::AuctionTimeCurveModel;


	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: DeFiComposableConfig + frame_system::Config {
		/// bank. vault owned - can transfer, cannot mint
		type Currency: Transfer<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>
		+ Mutate<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>
		// used to check balances before any storage updates allowing acting without rollback
		+ Inspect<Self::AccountId, Balance = Self::Balance, AssetId = Self::AssetId>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type UnixTime: UnixTime;
		type OrderId: OrderIdLike;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		
	}

	#[pallet::error]
	pub enum Error<T> {
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {}



	#[pallet::storage]
	#[pallet::getter(fn orders_index)]
	pub type OrdersIndex<T: Config> = StorageValue<_, T::OrderId, ValueQuery>;

}
