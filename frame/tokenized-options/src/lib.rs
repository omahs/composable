//! # Options Pallet
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

mod types;
mod validation;
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
	use crate::types::*;
	use crate::validation::*;
	use crate::weights::*;
	use codec::Codec;
	use composable_support::validation::Validated;
	use composable_traits::{
		currency::{CurrencyFactory, RangeId},
		defi::DeFiComposableConfig,
		swap_bytes::SwapBytes,
		tokenized_options::*,
		vault::{CapabilityVault, Deposit as Duration, Vault, VaultConfig},
	};
	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
			fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
			Time,
		},
		transactional, PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_runtime::{
		traits::{
			AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned, CheckedAdd, CheckedDiv,
			CheckedMul, CheckedSub, One, Saturating, Zero,
		},
		ArithmeticError, DispatchError, FixedPointNumber, FixedPointOperand, Perquintill,
	};
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

		/// tokenized_options PalletId
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Type of time moment. We use swap_bytes to store this type in big-endian format
		/// and take advantage of the fact that storage keys are stored in lexical order.
		type Moment: SwapBytes + AtLeast32Bit + Parameter + Copy + MaxEncodedLen;

		/// The time provider.
		type Time: Time<Moment = Self::Moment>;

		/// Option IDs generator
		type CurrencyFactory: CurrencyFactory<AssetIdOf<Self>>;

		/// PICA management
		type NativeCurrency: NativeTransfer<AccountIdOf<Self>, Balance = BalanceOf<Self>>
			+ NativeInspect<AccountIdOf<Self>, Balance = BalanceOf<Self>>;

		/// Options and other assets management
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
	//		Internal Pallet Types
	// ----------------------------------------------------------------------------------------------------
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type AssetIdOf<T> = <T as DeFiComposableConfig>::MayBeAssetId;
	pub type BalanceOf<T> = <T as DeFiComposableConfig>::Balance;
	pub type MomentOf<T> = <T as Config>::Moment;
	pub type VaultIdOf<T> = <T as Config>::VaultId;
	pub type VaultOf<T> = <T as Config>::Vault;
	pub type VaultConfigOf<T> = VaultConfig<AccountIdOf<T>, AssetIdOf<T>>;
	pub type OptionConfigOf<T> = OptionConfig<AssetIdOf<T>, BalanceOf<T>, MomentOf<T>>;

	// ----------------------------------------------------------------------------------------------------
	//		Storage
	// ----------------------------------------------------------------------------------------------------
	/// Maps and asset_id to its vault_id
	#[pallet::storage]
	#[pallet::getter(fn asset_id_to_vault_id)]
	pub type AssetToVault<T: Config> = StorageMap<_, Blake2_128Concat, AssetIdOf<T>, VaultIdOf<T>>;

	/// Maps an option_id to the option struct
	#[pallet::storage]
	#[pallet::getter(fn option_id_to_option)]
	pub type OptionIdToOption<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetIdOf<T>, OptionToken<T>>;

	/// Maps an account_id and an option_id to the user's provided collateral
	#[pallet::storage]
	#[pallet::getter(fn sellers)]
	pub type Sellers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AccountIdOf<T>,
		Blake2_128Concat,
		AssetIdOf<T>,
		SellerPosition<T>,
		OptionQuery,
	>;

	/// Maps a timestamp and an option_id to its currently active window_type.
	/// Scheduler is a timestamp-ordered list
	#[pallet::storage]
	pub(crate) type Scheduler<T: Config> =
		StorageDoubleMap<_, Identity, MomentOf<T>, Identity, AssetIdOf<T>, WindowType>;

	// ----------------------------------------------------------------------------------------------------
	//		Events
	// ----------------------------------------------------------------------------------------------------
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatedAssetVault { vault_id: VaultIdOf<T>, asset_id: AssetIdOf<T> },
		CreatedOption { option_id: AssetIdOf<T>, option_config: OptionConfigOf<T> },
		SellOption { seller: AccountIdOf<T>, option_amount: BalanceOf<T>, option_id: AssetIdOf<T> },
		BuyOption { buyer: AccountIdOf<T>, option_amount: BalanceOf<T>, option_id: AssetIdOf<T> },
	}

	// ----------------------------------------------------------------------------------------------------
	//		Errors
	// ----------------------------------------------------------------------------------------------------
	#[pallet::error]
	pub enum Error<T> {
		AssetVaultDoesNotExists,
		AssetVaultAlreadyExists,
		OptionIdNotFound,
		UserHasNotEnoughFundsToDeposit,
		DepositIntoVaultFailed,
		NotIntoDepositWindow,
		NotIntoPurchaseWindow,
		NotIntoExerciseWindow,
		NotIntoWithdrawWindow,
	}

	// ----------------------------------------------------------------------------------------------------
	//		Hooks
	// ----------------------------------------------------------------------------------------------------

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		/// At each block we perform timestamp checks to update the Scheduler
		fn on_idle(_n: T::BlockNumber, _remaining_weight: Weight) -> Weight {
			let now = T::Time::now();

			while let Some((moment, option_id, option_type)) = <Scheduler<T>>::iter().next() {
				let moment = moment.swap_bytes();

				if now < moment {
					break;
				}

				<Scheduler<T>>::remove(&moment, &option_id);

				match option_type {
					WindowType::Deposit => Self::option_deposit_start(option_id),
					WindowType::Purchase => Self::option_purchase_start(option_id),
					WindowType::Exercise => Self::option_exercise_start(option_id),
					WindowType::Withdraw => Self::option_withdraw_start(option_id),
					WindowType::End => Self::option_end(option_id),
				}
				.unwrap();
			}

			10_000
		}
	}

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
			origin: OriginFor<T>,
			config: VaultConfig<AccountIdOf<T>, AssetIdOf<T>>,
		) -> DispatchResult {
			// Check if it's protocol to call the exstrinsic
			let _from = ensure_signed(origin)?;

			let vault_id = Self::do_create_asset_vault(&config)?;

			Self::deposit_event(Event::CreatedAssetVault { vault_id, asset_id: config.asset_id });

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::create_option())]
		pub fn create_option(
			origin: OriginFor<T>,
			option_config: OptionConfigOf<T>,
		) -> DispatchResult {
			// Check if it's protocol to call the exstrinsic
			let _from = ensure_signed(origin)?;

			let option_id = <Self as TokenizedOptions>::create_option(option_config.clone())?;

			Self::deposit_event(Event::CreatedOption { option_id, option_config });

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::sell_option())]
		pub fn sell_option(
			origin: OriginFor<T>,
			option_amount: BalanceOf<T>,
			option_id: AssetIdOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			<Self as TokenizedOptions>::sell_option(&from, option_amount, option_id)?;

			Self::deposit_event(Event::SellOption { seller: from, option_amount, option_id });

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::buy_option())]
		pub fn buy_option(
			origin: OriginFor<T>,
			option_amount: BalanceOf<T>,
			option_id: AssetIdOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			<Self as TokenizedOptions>::buy_option(&from, option_amount, option_id)?;

			Self::deposit_event(Event::BuyOption { buyer: from, option_amount, option_id });

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
		type OptionToken = OptionToken<T>;
		type OptionConfig = OptionConfigOf<T>;

		fn create_option(
			option_config: Self::OptionConfig,
		) -> Result<Self::AssetId, DispatchError> {
			match Validated::new(option_config) {
				Ok(validated_option_config) => Self::do_create_option(validated_option_config),
				Err(_) => Err(DispatchError::from(Error::<T>::OptionIdNotFound)),
			}
		}

		fn sell_option(
			from: &Self::AccountId,
			option_amount: Self::Balance,
			option_id: Self::AssetId,
		) -> Result<(), DispatchError> {
			ensure!(OptionIdToOption::<T>::contains_key(option_id), Error::<T>::OptionIdNotFound);

			Self::do_sell_option(&from, option_amount, option_id)?;

			Ok(())
		}

		fn buy_option(
			from: &Self::AccountId,
			option_amount: Self::Balance,
			option_id: Self::AssetId,
		) -> Result<(), DispatchError> {
			ensure!(OptionIdToOption::<T>::contains_key(option_id), Error::<T>::OptionIdNotFound);

			Self::do_buy_option(&from, option_amount, option_id)?;

			Ok(())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Internal Pallet Functions
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> Pallet<T> {
		// Protocol account for a particular asset
		fn account_id(_asset: AssetIdOf<T>) -> AccountIdOf<T> {
			T::PalletId::get().into_sub_account(_asset)
		}

		#[transactional]
		fn do_create_asset_vault(
			config: &VaultConfig<AccountIdOf<T>, AssetIdOf<T>>,
		) -> Result<VaultIdOf<T>, DispatchError> {
			let asset_id = config.asset_id;
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
		fn do_create_option(
			option_config: Validated<OptionConfigOf<T>, ValidateOptionDoesNotExist<T>>,
		) -> Result<AssetIdOf<T>, DispatchError> {
			// Generate new option_id for the option token
			let option_id = T::CurrencyFactory::create(RangeId::LP_TOKENS)?;

			let option = OptionToken {
				base_asset_id: option_config.base_asset_id,
				quote_asset_id: option_config.quote_asset_id,
				base_asset_strike_price: option_config.base_asset_strike_price,
				option_type: option_config.option_type,
				exercise_type: option_config.exercise_type,
				expiring_date: option_config.expiring_date,
				base_asset_amount_per_option: option_config.base_asset_amount_per_option,
				total_issuance_seller: option_config.total_issuance_seller,
				total_issuance_buyer: option_config.total_issuance_buyer,
				epoch: option_config.epoch,
			};

			// Add option_id to corresponding option
			OptionIdToOption::<T>::insert(option_id, option);

			Ok(option_id)
		}

		#[transactional]
		fn do_sell_option(
			from: &AccountIdOf<T>,
			option_amount: BalanceOf<T>,
			option_id: AssetIdOf<T>,
		) -> Result<(), DispatchError> {
			// Get option
			let option =
				Self::option_id_to_option(option_id).ok_or(Error::<T>::OptionIdNotFound)?;

			// Check if deposits are allowed
			let now = T::Time::now();
			ensure!(
				option.epoch.window_type(now).unwrap() == WindowType::Deposit,
				Error::<T>::NotIntoDepositWindow
			);

			// Different behaviors based on Call or Put option
			let mut asset_id = option.base_asset_id;
			let mut asset_amount = option.base_asset_amount_per_option * option_amount;

			if option.option_type == OptionType::Put {
				asset_id = option.quote_asset_id;
				asset_amount = option.base_asset_strike_price * option_amount;
			};

			// Get vault_id for depositing collateral
			let vault_id =
				Self::asset_id_to_vault_id(asset_id).ok_or(Error::<T>::AssetVaultDoesNotExists)?;

			// Get pallet account
			let protocol_account = Self::account_id(asset_id);

			// Get user's position if it already has one, otherwise create a default
			let position = Self::sellers(from, option_id).unwrap();

			// Check if user has funds for collateral
			ensure!(
				<T as Config>::MultiCurrency::can_withdraw(asset_id, &from, asset_amount)
					.into_result()
					.is_ok(),
				Error::<T>::UserHasNotEnoughFundsToDeposit
			);

			// Check if user can deposit
			ensure!(
				<T as Config>::MultiCurrency::can_deposit(
					asset_id,
					&protocol_account,
					asset_amount
				)
				.into_result()
				.is_ok(),
				Error::<T>::DepositIntoVaultFailed
			);

			// Transfer collateral to protocol account
			// THIS SHOULD BE DONE AT THE END. NOT POSSIBLE RIGHT NOW BUT WILL BE REFACTORED
			<T as Config>::MultiCurrency::transfer(
				asset_id,
				&from,
				&protocol_account,
				asset_amount,
				true,
			)?;

			// Protocol account deposits into the vault and keep shares_amount
			// THIS SHOULD BE DONE AT THE END. NOT POSSIBLE RIGHT NOW BUT WILL BE REFACTORED
			let shares_amount = T::Vault::deposit(&vault_id, &protocol_account, asset_amount)?;

			// Add option amount to position
			// THIS SHOULD BE DONE BEFORE TRANSFER AND DEPOSIT. NOT POSSIBLE RIGHT NOW BUT WILL BE REFACTORED
			let new_option_amount = position
				.option_amount
				.checked_add(&option_amount)
				.ok_or(ArithmeticError::Overflow)?;

			// Add shares amount to position
			// THIS SHOULD BE DONE BEFORE TRANSFER AND DEPOSIT. NOT POSSIBLE RIGHT NOW BUT WILL BE REFACTORED
			let new_shares_amount = position
				.shares_amount
				.checked_add(&shares_amount)
				.ok_or(ArithmeticError::Overflow)?;

			Ok(())
		}

		#[transactional]
		fn do_buy_option(
			from: &AccountIdOf<T>,
			option_amount: BalanceOf<T>,
			option_id: AssetIdOf<T>,
		) -> Result<(), DispatchError> {
			Ok(())
		}
	}
}

// ----------------------------------------------------------------------------------------------------
//		Unit Tests
// ----------------------------------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {}
