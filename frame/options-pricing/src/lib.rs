#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
	not(test),
	deny(
		clippy::disallowed_methods,
		clippy::disallowed_types,
		clippy::indexing_slicing,
		clippy::todo,
		clippy::unwrap_used,
		clippy::panic
	)
)] // allow in tests
#![deny(
	dead_code,
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

pub use crate::weights::WeightInfo;
mod types;
mod weights;
// #[allow(unused_imports)]
#[cfg(test)]
mod mocks;

#[cfg(test)]
#[allow(dead_code)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(dead_code)]
pub mod pallet {
	// ----------------------------------------------------------------------------------------------------
	//		Imports and Dependencies
	// ----------------------------------------------------------------------------------------------------
	use crate::{types::*, weights::*};

	use codec::Codec;
	use composable_support::validation::Validated;
	use composable_traits::{
		currency::{CurrencyFactory, LocalAssets, RangeId},
		defi::DeFiComposableConfig,
		options_pricing::*,
		oracle::Oracle,
		swap_bytes::{SwapBytes, Swapped},
		tokenized_options::*,
		vault::{CapabilityVault, Deposit as Duration, Vault, VaultConfig},
	};

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		storage::{bounded_btree_map::BoundedBTreeMap, bounded_btree_set::BoundedBTreeSet},
		traits::{
			fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
			fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
			EnsureOrigin, Time,
		},
		transactional, PalletId,
	};

	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_core::H256;
	use sp_runtime::{
		helpers_128bit::multiply_by_rational,
		traits::{
			AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned, BlakeTwo256, CheckedAdd,
			CheckedDiv, CheckedMul, CheckedSub, Convert, One, Saturating, Zero,
		},
		ArithmeticError, DispatchError, FixedI128, FixedPointNumber, FixedPointOperand,
		Perquintill,
	};
	use sp_std::cmp::min;

	use sp_std::{collections::btree_map::BTreeMap, fmt::Debug};
	// ----------------------------------------------------------------------------------------------------
	//		Declaration Of The Pallet Type
	// ----------------------------------------------------------------------------------------------------
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// ----------------------------------------------------------------------------------------------------
	//		Config Trait
	// ----------------------------------------------------------------------------------------------------
	#[pallet::config]
	pub trait Config: frame_system::Config + DeFiComposableConfig {
		#[allow(missing_docs)]
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		/// The id used as `AccountId` for the pallet.
		/// This should be unique across all pallets to avoid name collisions.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Type of time moment. We use [`SwapBytes`] trait to store this type in
		/// big endian format and take advantage of the fact that storage keys are
		/// stored in lexical order.
		type Moment: SwapBytes + AtLeast32Bit + Parameter + Copy + MaxEncodedLen;

		/// The Unix time provider.
		type Time: Time<Moment = MomentOf<Self>>;

		/// Oracle pallet to retrieve prices expressed in USDT.
		type Oracle: Oracle<AssetId = AssetIdOf<Self>, Balance = BalanceOf<Self>>;

		/// Protocol Origin that can create vaults and options.
		type ProtocolOrigin: EnsureOrigin<Self::Origin>;

		/// Used for option tokens and other assets management.
		type Assets: Transfer<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ Mutate<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ MutateHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ Inspect<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ InspectHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>;

		/// Trait used to convert from this pallet `Balance` type to `u128`.
		type ConvertBalanceToDecimal: Convert<BalanceOf<Self>, Decimal>
			+ Convert<Decimal, BalanceOf<Self>>;

		type ConvertMomentToDecimal: Convert<MomentOf<Self>, Decimal>;
	}

	// ----------------------------------------------------------------------------------------------------
	//		Internal Pallet Types
	// ----------------------------------------------------------------------------------------------------
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type AssetIdOf<T> = <T as DeFiComposableConfig>::MayBeAssetId;
	pub type BalanceOf<T> = <T as DeFiComposableConfig>::Balance;
	pub type AssetsOf<T> = <T as Config>::Assets;
	pub type MomentOf<T> = <T as Config>::Moment;
	pub type OracleOf<T> = <T as Config>::Oracle;
	pub type OptionIdOf<T> = AssetIdOf<T>;
	pub type BlackScholesParamsOf<T> = BlackScholesParams<AssetIdOf<T>, BalanceOf<T>, MomentOf<T>>;
	pub type Decimal = FixedI128;

	// ----------------------------------------------------------------------------------------------------
	//		Storage
	// ----------------------------------------------------------------------------------------------------
	#[pallet::storage]
	#[pallet::getter(fn interest_rate_index)]
	#[allow(clippy::disallowed_types)]
	pub type InterestRateIndex<T: Config> = StorageValue<_, Decimal, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn latest_iv)]
	pub type LatestIV<T: Config> = StorageMap<_, Blake2_128Concat, OptionIdOf<T>, Decimal>;

	#[pallet::storage]
	#[pallet::getter(fn latest_price)]
	pub type LatestPrice<T: Config> = StorageMap<_, Blake2_128Concat, OptionIdOf<T>, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn snapshots)]
	pub type Snapshots<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OptionIdOf<T>,
		Blake2_128Concat,
		MomentOf<T>,
		Snapshot<T>,
	>;

	// ----------------------------------------------------------------------------------------------------
	//		Events
	// ----------------------------------------------------------------------------------------------------
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		InterestRateIndexUpdated { interest_rate: Decimal },
	}

	// ----------------------------------------------------------------------------------------------------
	//		Errors
	// ----------------------------------------------------------------------------------------------------
	#[pallet::error]
	pub enum Error<T> {
		FailedConversion,
	}

	// ----------------------------------------------------------------------------------------------------
	//		Hooks
	// ----------------------------------------------------------------------------------------------------

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	// ----------------------------------------------------------------------------------------------------
	//		Extrinsics
	// ----------------------------------------------------------------------------------------------------

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::calculate_option_price())]
		pub fn calculate_option_price(
			origin: OriginFor<T>,
			params: BlackScholesParamsOf<T>,
		) -> DispatchResult {
			// Check if it's protocol to call the extrinsic
			T::ProtocolOrigin::ensure_origin(origin)?;

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::calculate_option_greeks())]
		pub fn calculate_option_greeks(
			origin: OriginFor<T>,
			params: BlackScholesParamsOf<T>,
		) -> DispatchResult {
			// Check if it's protocol to call the extrinsic
			T::ProtocolOrigin::ensure_origin(origin)?;

			Ok(())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		OptionsPricing Trait
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> OptionsPricing for Pallet<T> {
		type AssetId = AssetIdOf<T>;
		type Balance = BalanceOf<T>;
		type Moment = MomentOf<T>;

		#[transactional]
		fn calculate_option_price(
			params: BlackScholesParamsOf<T>,
		) -> Result<Self::Balance, DispatchError> {
			Self::do_calculate_option_price(params)
		}


	}

	// ----------------------------------------------------------------------------------------------------
	//		Internal Pallet Functions
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> Pallet<T> {
		fn do_calculate_option_price(
			params: BlackScholesParamsOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			// Get interest rate index, annualized expiry date and converted prices
			let interest_rate = Self::interest_rate_index();
			let time_annualized = Self::get_expiry_time_annualized(params.expiring_date)?;
			let strike_price = T::ConvertBalanceToDecimal::convert(params.base_asset_strike_price);
			let spot_price = T::ConvertBalanceToDecimal::convert(params.base_asset_spot_price);

			// Get volatility for option's asset
			let iv: Decimal = Decimal::from_float(200.5); // TODO

			let option_price = Self::black_scholes(
				strike_price,
				spot_price,
				time_annualized,
				interest_rate,
				iv,
				params.option_type,
			)?;

			Ok(option_price)
		}

		fn black_scholes(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			iv: Decimal,
			option_type: OptionType,
		) -> Result<BalanceOf<T>, DispatchError> {
			let (d1, d2) = Self::calculate_d1_d2(
				strike_price,
				spot_price,
				time_annualized,
				interest_rate,
				iv,
			)?;

			let option_price = match option_type {
				OptionType::Call => Self::calculate_call_price(
					strike_price,
					spot_price,
					time_annualized,
					interest_rate,
					d1,
					d2,
				)?,
				OptionType::Put => Self::calculate_put_price(
					strike_price,
					spot_price,
					time_annualized,
					interest_rate,
					d1,
					d2,
				)?,
			};

			Ok((1000u128 * 10u128.pow(12)).into())
		}

		fn get_expiry_time_annualized(expiry_date: MomentOf<T>) -> Result<Decimal, Error<T>> {
			let now = T::Time::now();
			let seconds_to_expiry = T::ConvertMomentToDecimal::convert(expiry_date - now);
			seconds_to_expiry
				.checked_div(&SECONDS_PER_YEAR)
				.ok_or(Error::<T>::FailedConversion)
		}

		fn cumulative_normal_distribution(value: Decimal) -> Result<Decimal, DispatchError> {
			Ok(Decimal::from_inner(10.into()))
		}

		fn calculate_d1_d2(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			iv: Decimal,
		) -> Result<(Decimal, Decimal), DispatchError> {
			// let a = Decimal::sqrt(time_annualized).ok_or(ArithmeticError::Underflow)?;
			// let a = iv.checked_mul(&a).ok_or(ArithmeticError::Overflow)?;

			// let b = spot_price.checked_div(&strike_price).ok_or(ArithmeticError::Overflow)?;
			// let b = Decimal::log(b).ok_or(ArithmeticError::Underflow)?;

			// let c = iv.saturating_pow(2);
			// let c = c.checked_div(&Decimal::from_inner(2.into())).ok_or(ArithmeticError::Underflow)?;
			// let c = c.checked_add(&interest_rate).ok_or(ArithmeticError::Overflow)?;
			// let c = c.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;

			// let d1 = b.checked_add(&c).ok_or(ArithmeticError::Overflow)?;
			// let d1 = d1.checked_div(&a).ok_or(ArithmeticError::Overflow)?;

			// let d2 = d1.checked_sub(&a).ok_or(ArithmeticError::Underflow)?;

			// Ok((d1, d2))
			Ok((Decimal::from_inner(10.into()), Decimal::from_inner(10.into())))
		}

		fn calculate_call_price(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			d1: Decimal,
			d2: Decimal,
		) -> Result<Decimal, DispatchError> {
			// let nd1 = Self::cumulative_normal_distribution(d1)?;
			// let a = spot_price.checked_mul(&nd1).ok_or(ArithmeticError::Overflow)?;

			// let nd2 = Self::cumulative_normal_distribution(d2)?;
			// let exp =
			// 	-interest_rate.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;
			// let exp = Decimal::exp(&exp).ok_or(ArithmeticError::Overflow)?;
			// let b = strike_price.checked_mul(&nd2).ok_or(ArithmeticError::Overflow)?;
			// let b = b.checked_mul(&exp).ok_or(ArithmeticError::Overflow)?;

			// Ok(a.checked_sub(&b))
			Ok(1.into())
		}

		fn calculate_put_price(
			strike_price: Decimal,
			spot_price: Decimal,
			time_annualized: Decimal,
			interest_rate: Decimal,
			d1: Decimal,
			d2: Decimal,
		) -> Result<Decimal, DispatchError> {
			// let nd2 = Self::cumulative_normal_distribution(-d2)?;
			// let exp =
			// 	-interest_rate.checked_mul(&time_annualized).ok_or(ArithmeticError::Overflow)?;
			// let exp = Decimal::exp(&exp).ok_or(ArithmeticError::Overflow)?;
			// let b = strike_price.checked_mul(&nd2).ok_or(ArithmeticError::Overflow)?;
			// let b = b.checked_mul(&exp).ok_or(ArithmeticError::Overflow)?;

			// let nd1 = Self::cumulative_normal_distribution(-d1)?;
			// let a = spot_price.checked_mul(&nd1).ok_or(ArithmeticError::Overflow)?;

			// Ok(b.checked_sub(&a))
			Ok(1.into())
		}
	}
}
