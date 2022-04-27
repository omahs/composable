//! # Options Pallet

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

mod weights;

pub use pallet::*;

#[frame_support::pallet]
#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(dead_code)]
pub mod pallet {
	// ----------------------------------------------------------------------------------------------------
	//		Imports and Dependencies
	// ----------------------------------------------------------------------------------------------------
	use crate::weights::WeightInfo;
	use codec::{Codec, FullCodec};
	use composable_traits::currency::{AssetIdLike, BalanceLike, CurrencyFactory, RangeId};
	use composable_traits::defi::DeFiComposableConfig;
	use composable_traits::time::Timestamp;
	use composable_traits::tokenized_options::*;
	use frame_support::traits::UnixTime;
	use sp_runtime::DispatchError;

	use composable_traits::vault::{CapabilityVault, Deposit as Duration, Vault, VaultConfig};
	use frame_support::pallet_prelude::*;
	use frame_support::{
		traits::{
			fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
			fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
		},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use frame_system::{ensure_root, ensure_signed};
	use sp_runtime::{
		traits::{
			AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub, Zero,
		},
		Perquintill,
	};
	use sp_std::fmt::Debug;
	use std::collections::BTreeMap;

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

		/// tokenized_options PalletId
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type WeightInfo: WeightInfo;

		// type UnixTime = UnixTime;

		type Timestamp: Default
			+ Clone
			+ Copy
			+ Debug
			+ FullCodec
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize
			+ PartialEq
			+ TypeInfo;

		type CurrencyFactory: CurrencyFactory<AssetIdOf<Self>>;

		type NativeCurrency: NativeTransfer<AccountIdOf<Self>, Balance = BalanceOf<Self>>
			+ NativeInspect<AccountIdOf<Self>, Balance = BalanceOf<Self>>;

		type MultiCurrency: Transfer<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ Mutate<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ MutateHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ InspectHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>;

		type VaultId: Clone + Codec + MaxEncodedLen + Debug + PartialEq + Default + Parameter;

		type Vault: CapabilityVault<
			AssetId = AssetIdOf<Self>,
			Balance = BalanceOf<Self>,
			AccountId = AccountIdOf<Self>,
			VaultId = VaultIdOf<Self>,
		>;
	}

	// ----------------------------------------------------------------------------------------------------
	//		Helper Pallet Types
	// ----------------------------------------------------------------------------------------------------
	#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
	pub enum OptionType {
		Call,
		Put,
	}

	#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
	pub enum ExerciseType {
		European,
		American,
	}

	#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
	pub enum WindowType {
		Deposit,
		Purchase,
		Exercise,
		Withdraw,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct OptionToken<AssetId, Balance, Timestamp> {
		pub base_asset_id: AssetId,
		pub quote_asset_id: AssetId,
		pub base_asset_strike_price: Balance,
		// pub quote_asset_strike_price: Balance, // Assume stablecoin as quote asset right now, so always 1
		pub option_type: OptionType,
		pub exercise_type: ExerciseType,
		pub expiring_date: Timestamp,
		pub base_asset_amount_per_option: Balance,
		// pub quote_asset_amount_per_option: Balance, // Assume stablecoin as quote asset right now, so always 1
		pub total_issuance_seller: Balance,
		pub total_issuance_buyer: Balance,
		pub epoch: Epoch<TimeWindow<Timestamp>>,
	}

	#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
	pub struct TimeWindow<Timestamp> {
		pub start: Timestamp,
		pub end: Timestamp,
		pub window_type: WindowType,
	}

	#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
	pub struct Epoch<TimeWindow> {
		pub deposit_window: TimeWindow,
		pub purchase_window: TimeWindow,
		pub exercise_window: TimeWindow,
		pub withdraw_window: TimeWindow,
	}

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type AssetIdOf<T> = <T as DeFiComposableConfig>::MayBeAssetId;
	type BalanceOf<T> = <T as DeFiComposableConfig>::Balance;
	type VaultIdOf<T> = <T as Config>::VaultId;
	type TimestampOf<T> = <T as Config>::Timestamp;
	type VaultOf<T> = <T as Config>::Vault;
	type OptionOf<T> = OptionToken<AssetIdOf<T>, BalanceOf<T>, TimestampOf<T>>;

	// ----------------------------------------------------------------------------------------------------
	//		Storage
	// ----------------------------------------------------------------------------------------------------
	#[pallet::storage]
	#[pallet::getter(fn asset_id_to_vault_id)]
	pub type AssetToVault<T: Config> = StorageMap<_, Blake2_128Concat, AssetIdOf<T>, VaultIdOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn option_id_to_option)]
	pub type OptionIdToOption<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetIdOf<T>, OptionOf<T>>;

	// ----------------------------------------------------------------------------------------------------
	//		Events
	// ----------------------------------------------------------------------------------------------------
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatedAssetVault { vault_id: VaultIdOf<T>, asset_id: AssetIdOf<T> },
		CreatedOption { option_id: AssetIdOf<T>, option: OptionOf<T> },
		SellOption { who: AccountIdOf<T>, amount: BalanceOf<T>, option_id: AssetIdOf<T> },
		BuyOption { who: AccountIdOf<T>, amount: BalanceOf<T>, option_id: AssetIdOf<T> },
	}

	// ----------------------------------------------------------------------------------------------------
	//		Errors
	// ----------------------------------------------------------------------------------------------------
	#[pallet::error]
	pub enum Error<T> {
		AssetVaultAlreadyExists,
		OptionIdNotFound,
	}

	// ----------------------------------------------------------------------------------------------------
	//		Hooks
	// ----------------------------------------------------------------------------------------------------
	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	// ----------------------------------------------------------------------------------------------------
	//		Genesis Config
	// ----------------------------------------------------------------------------------------------------

	// ----------------------------------------------------------------------------------------------------
	//		Extrinsics
	// ----------------------------------------------------------------------------------------------------
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_asset_vault())]
		pub fn create_asset_vault(
			_origin: OriginFor<T>,
			_config: VaultConfig<AccountIdOf<T>, AssetIdOf<T>>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(_origin)?;

			let vault_id = Self::do_create_asset_vault(&_config)?;

			Self::deposit_event(Event::CreatedAssetVault { vault_id, asset_id: _config.asset_id });

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::create_option())]
		pub fn create_option(
			_origin: OriginFor<T>,
			_option: OptionOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(_origin)?;

			let option_id = <Self as TokenizedOptions>::create_option(from, &_option)?;

			Self::deposit_event(Event::CreatedOption { option_id, option: _option });

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::sell_option())]
		pub fn sell_option(
			_origin: OriginFor<T>,
			_amount: BalanceOf<T>,
			_option_id: AssetIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(_origin)?;

			<Self as TokenizedOptions>::sell_option(&from, _amount, _option_id)?;

			Self::deposit_event(Event::SellOption {
				who: from,
				amount: _amount,
				option_id: _option_id,
			});

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::buy_option())]
		pub fn buy_option(
			_origin: OriginFor<T>,
			_amount: BalanceOf<T>,
			_option_id: AssetIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(_origin)?;

			<Self as TokenizedOptions>::buy_option(from, _amount, _option_id)?;

			// Self::deposit_event(Event::BuyOption);

			Ok(().into())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		TokenizedOptions Trait
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> TokenizedOptions for Pallet<T> {
		type AccountId = AccountIdOf<T>;
		type AssetId = AssetIdOf<T>;
		type Balance = BalanceOf<T>;
		type OptionToken = OptionOf<T>;

		fn create_option(
			_from: Self::AccountId,
			_option: &Self::OptionToken,
		) -> Result<Self::AssetId, DispatchError> {
			let option_id = Self::do_create_option(&_option).unwrap();

			Ok(option_id)
		}

		fn sell_option(
			_from: &Self::AccountId,
			_amount: Self::Balance,
			_option_id: Self::AssetId,
		) -> Result<(), DispatchError> {
			ensure!(OptionIdToOption::<T>::contains_key(_option_id), Error::<T>::OptionIdNotFound);

			Self::do_sell_option(&_from, _amount, _option_id);

			Ok(())
		}

		fn buy_option(
			_from: Self::AccountId,
			_amount: Self::Balance,
			_option_id: Self::AssetId,
		) -> Result<(), DispatchError> {
			ensure!(OptionIdToOption::<T>::contains_key(_option_id), Error::<T>::OptionIdNotFound);

			Self::do_buy_option(_from, _amount, _option_id);

			Ok(())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Internal Functions
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> Pallet<T> {
		fn account_id(_asset: AssetIdOf<T>) -> AccountIdOf<T> {
			T::PalletId::get().into_sub_account(_asset)
		}

		#[transactional]
		fn do_create_asset_vault(
			_config: &VaultConfig<AccountIdOf<T>, AssetIdOf<T>>,
		) -> Result<VaultIdOf<T>, DispatchError> {
			let asset_id = _config.asset_id;
			ensure!(
				!AssetToVault::<T>::contains_key(asset_id),
				Error::<T>::AssetVaultAlreadyExists
			);

			// Get pallet account for the asset
			let account_id = Self::account_id(asset_id);

			// Create new vault for the asset
			let asset_vault_id: T::VaultId = T::Vault::create(
				Duration::Existential,
				VaultConfig {
					asset_id,
					manager: account_id,
					reserved: Perquintill::one(),
					strategies: BTreeMap::new(),
				},
			)?;

			// Add asset to the corresponding asset vault
			AssetToVault::<T>::insert(asset_id, &asset_vault_id);

			Ok(asset_vault_id)
		}

		#[transactional]
		fn do_create_option(_option: &OptionOf<T>) -> Result<AssetIdOf<T>, DispatchError> {
			// Check if option already exists (how?)

			// Generate new option_id for the option token
			let option_id = T::CurrencyFactory::create(RangeId::LP_TOKENS)?;

			// Add option_id to corresponding option
			OptionIdToOption::<T>::insert(option_id, _option);

			Ok(option_id)
		}

		#[transactional]
		fn do_sell_option(
			_from: &AccountIdOf<T>,
			_amount: BalanceOf<T>,
			_option_id: AssetIdOf<T>,
		) -> Result<(), DispatchError> {
			// Get pallet option_account
			let account_id = Self::account_id(_option_id);

			let option =
				Self::option_id_to_option(_option_id).ok_or(Error::<T>::OptionIdNotFound)?;

			// Do stuff with option (based on option's attributes)

			// Right now I'm simply calling this since there is no a fixed design for options yet
			// and I'm setting the test env for this function
			<T as Config>::MultiCurrency::mint_into(_option_id, _from, _amount);

			Ok(())
		}

		#[transactional]
		fn do_buy_option(
			_from: AccountIdOf<T>,
			_amount: BalanceOf<T>,
			_option_id: AssetIdOf<T>,
		) -> Result<(), DispatchError> {
			// Get pallet option_account
			let account_id = Self::account_id(_option_id);

			let option =
				Self::option_id_to_option(_option_id).ok_or(Error::<T>::OptionIdNotFound)?;

			Ok(())
		}
	}
}

// ----------------------------------------------------------------------------------------------------
//		Unit Tests
// ----------------------------------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {}
