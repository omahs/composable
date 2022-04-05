//! # Options Pallet

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod currency;

mod weights;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	// ----------------------------------------------------------------------------------------------------
	//		Imports and Dependencies
	// ----------------------------------------------------------------------------------------------------
	use crate::weights::WeightInfo;
	use codec::{Codec, FullCodec};
	use composable_traits::currency::{AssetIdLike, BalanceLike, CurrencyFactory, RangeId};
	use composable_traits::defi::DeFiComposableConfig;
	use composable_traits::tokenized_options::{
		ExerciseType, OptionToken, OptionType, TokenizedOptions,
	};
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
	use frame_system::ensure_signed;
	use frame_system::pallet_prelude::*;
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
	pub trait Config: frame_system::Config {
		#[allow(missing_docs)]
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Balance: Default
			+ Parameter
			+ Codec
			+ MaxEncodedLen
			+ Copy
			+ Ord
			+ CheckedAdd
			+ CheckedSub
			+ CheckedMul
			+ AtLeast32BitUnsigned
			+ Zero;

		type AssetId: FullCodec
			+ MaxEncodedLen
			+ Eq
			+ PartialEq
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ Default
			+ TypeInfo;

		/// tokenized_options PalletId
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type WeightInfo: WeightInfo;

		type CurrencyFactory: CurrencyFactory<AssetIdOf<Self>>;

		type NativeCurrency: NativeTransfer<AccountIdOf<Self>, Balance = BalanceOf<Self>>
			+ NativeInspect<AccountIdOf<Self>, Balance = BalanceOf<Self>>;

		type MultiCurrency: Transfer<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ Mutate<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ MutateHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ InspectHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>;

		type VaultId: Clone + Codec + MaxEncodedLen + Debug + PartialEq + Default + Parameter;

		type AssetVault: CapabilityVault<
			AssetId = AssetIdOf<Self>,
			Balance = BalanceOf<Self>,
			AccountId = AccountIdOf<Self>,
			VaultId = VaultIdOf<Self>,
		>;
	}

	// ----------------------------------------------------------------------------------------------------
	//		Helper Pallet Types
	// ----------------------------------------------------------------------------------------------------
	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type AssetIdOf<T> = <T as Config>::AssetId;
	type BalanceOf<T> = <T as Config>::Balance;
	type VaultIdOf<T> = <T as Config>::VaultId;
	type OptionOf<T> = OptionToken<AssetIdOf<T>, BalanceOf<T>>;
	type AssetVaultOf<T> = <T as Config>::AssetVault;

	// ----------------------------------------------------------------------------------------------------
	//		Storage
	// ----------------------------------------------------------------------------------------------------

	#[pallet::storage]
	#[pallet::getter(fn option_id_counter)]
	pub type OptionIdCounter<T: Config> = StorageValue<_, AssetIdOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn option_id_to_option)]
	pub type OptionIdToOption<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetIdOf<T>, OptionOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn option_id_to_option_vault_id)]
	pub type OptionIdToOptionVaultId<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetIdOf<T>, VaultIdOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn asset_id_to_vault_id)]
	pub type AssetIdToVaultId<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetIdOf<T>, VaultIdOf<T>>;

	//  TODO: Maps each account to its minted options for a particular asset
	//  TODO: Maps each account to its collateral for a particular asset

	// ----------------------------------------------------------------------------------------------------
	//		Events
	// ----------------------------------------------------------------------------------------------------
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatedAssetVault,
		CreatedOptionVault,
		SellOption,
		BuyOption,
	}

	// ----------------------------------------------------------------------------------------------------
	//		Errors
	// ----------------------------------------------------------------------------------------------------
	#[pallet::error]
	pub enum Error<T> {
		VaultAlreadyExists,
		AssetDoesNotHaveVault,
	}

	// ----------------------------------------------------------------------------------------------------
	//		Hooks
	// ----------------------------------------------------------------------------------------------------

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	// ----------------------------------------------------------------------------------------------------
	//		Genesis Config
	// ----------------------------------------------------------------------------------------------------
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub options_counter: AssetIdOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { options_counter: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			OptionIdCounter::<T>::put(self.options_counter);
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Extrinsics
	// ----------------------------------------------------------------------------------------------------

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_asset_vault())]
		pub fn create_asset_vault(
			_origin: OriginFor<T>,
			_asset: AssetIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(_origin)?;

			let vault_id = Self::do_create_asset_vault(&_asset)?;

			Self::deposit_event(Event::CreatedAssetVault);

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::create_option_vault())]
		pub fn create_option_vault(
			_origin: OriginFor<T>,
			_amount: BalanceOf<T>,
			_option: OptionOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(_origin)?;

			let (option_id, option_vault_id) =
				<Self as TokenizedOptions>::create_option_vault(from, _amount, &_option)?;

			Self::deposit_event(Event::CreatedOptionVault);

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::sell_option())]
		pub fn sell_option(
			_origin: OriginFor<T>,
			_amount: BalanceOf<T>,
			_option_id: AssetIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(_origin)?;

			<Self as TokenizedOptions>::sell_option(from, _amount, _option_id)?;

			Self::deposit_event(Event::SellOption);

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

			Self::deposit_event(Event::BuyOption);

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
		type VaultId = VaultIdOf<T>;

		fn create_option_vault(
			_from: Self::AccountId,
			_amount: Self::Balance,
			_option: &OptionToken<Self::AssetId, Self::Balance>,
		) -> Result<(Self::AssetId, Self::VaultId), DispatchError> {
			let (option_id, option_vault_id) = Self::do_create_option_vault(&_option).unwrap();

			Ok((option_id, option_vault_id))
		}

		fn sell_option(
			_from: Self::AccountId,
			_amount: Self::Balance,
			_option_id: Self::AssetId,
		) -> Result<(), DispatchError> {
			Self::do_sell_option(_amount, _option_id).unwrap();

			// Save that "_from" deposited

			Ok(())
		}

		fn buy_option(
			_from: Self::AccountId,
			_amount: Self::Balance,
			_option_id: Self::AssetId,
		) -> Result<(), DispatchError> {
			Self::do_buy_option(_from, _amount, _option_id).unwrap();

			// Save that "_from" has bought

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

		fn do_create_asset_vault(_asset_id: &AssetIdOf<T>) -> Result<VaultIdOf<T>, DispatchError> {
			let asset_id = *_asset_id;
			// Get pallet asset_account
			let account_id = Self::account_id(asset_id);

			// Create new vault for depositing base asset
			let asset_vault_id: T::VaultId = T::AssetVault::create(
				Duration::Existential,
				VaultConfig {
					asset_id,
					manager: account_id,
					reserved: Perquintill::one(),
					strategies: BTreeMap::new(),
				},
			)?;

			// Add base_asset_id to the corresponding asset vault
			AssetIdToVaultId::<T>::insert(_asset_id, &asset_vault_id);

			Ok(asset_vault_id)
		}

		fn do_create_option_vault(
			_option: &OptionOf<T>,
		) -> Result<(AssetIdOf<T>, VaultIdOf<T>), DispatchError> {
			// Generate new option_id for the option token
			let option_id = T::CurrencyFactory::create(RangeId::LP_TOKENS)?;

			// Get pallet option_account
			let account_id = Self::account_id(option_id);

			// Create new vault to gather option tokens
			let option_vault_id: T::VaultId = T::AssetVault::create(
				Duration::Existential,
				VaultConfig {
					asset_id: option_id,
					manager: account_id,
					reserved: Perquintill::one(),
					strategies: BTreeMap::new(),
				},
			)?;

			// Add option_id to corresponding option
			OptionIdToOption::<T>::insert(option_id, _option.clone());

			// Add option_id to the corresponding token vault
			OptionIdToOptionVaultId::<T>::insert(option_id, &option_vault_id);

			Ok((option_id, option_vault_id))
		}

		fn do_sell_option(
			_amount: BalanceOf<T>,
			_option_id: AssetIdOf<T>,
		) -> Result<(), DispatchError> {
			// Get pallet option_account
			let account_id = Self::account_id(_option_id);

			T::MultiCurrency::mint_into(_option_id, &account_id, _amount)?;

			Ok(())
		}

		fn do_buy_option(
			_from: AccountIdOf<T>,
			_amount: BalanceOf<T>,
			_option_id: AssetIdOf<T>,
		) -> Result<(), DispatchError> {
			// Get pallet option_account
			let account_id = Self::account_id(_option_id);

			T::MultiCurrency::mint_into(_option_id, &_from, _amount)?;

			Ok(())
		}
	}
}

// ----------------------------------------------------------------------------------------------------
//		Unit Tests
// ----------------------------------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {}
