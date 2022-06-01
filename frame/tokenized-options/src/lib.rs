//! # Options Pallet
#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
#[allow(dead_code)]
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
	use crate::{types::*, validation::*, weights::*};

	use codec::Codec;
	use composable_support::validation::Validated;
	use composable_traits::{
		currency::{CurrencyFactory, RangeId},
		defi::DeFiComposableConfig,
		oracle::Oracle,
		swap_bytes::{SwapBytes, Swapped},
		tokenized_options::*,
		vault::{CapabilityVault, Deposit as Duration, Vault, VaultConfig},
	};
	use frame_support::{
		pallet_prelude::{ValueQuery, *},
		sp_runtime::traits::Hash,
		storage::{bounded_btree_map::BoundedBTreeMap, bounded_btree_set::BoundedBTreeSet},
		traits::{
			fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
			fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
			Time,
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

		/// Maximum number of options that can be created
		#[pallet::constant]
		type MaxOptionNumber: Get<u32>;

		/// Oracle pallet to retrieve prices
		type Oracle: Oracle<AssetId = AssetIdOf<Self>, Balance = BalanceOf<Self>>;

		/// Type of time moment. We use [`SwapBytes`] trait to store this type in
		/// big endian format and take advantage of the fact that storage keys are
		/// stored in lexical order.
		type Moment: SwapBytes + AtLeast32Bit + Parameter + Copy + MaxEncodedLen;

		/// The time provider
		type Time: Time<Moment = MomentOf<Self>>;

		type Convert: Convert<BalanceOf<Self>, u128> + Convert<u128, BalanceOf<Self>>;

		/// Option IDs generator
		type CurrencyFactory: CurrencyFactory<OptionIdOf<Self>>;

		/// PICA management
		type NativeCurrency: NativeTransfer<AccountIdOf<Self>, Balance = BalanceOf<Self>>
			+ NativeInspect<AccountIdOf<Self>, Balance = BalanceOf<Self>>;

		/// Option tokens and other assets management
		type MultiCurrency: Transfer<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ Mutate<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ MutateHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ Inspect<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
			+ InspectHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>;

		/// Vault IDs
		type VaultId: Clone + Copy + Codec + MaxEncodedLen + Debug + PartialEq + Default + Parameter;

		/// Vaults to collect collaterals
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
	pub type OracleOf<T> = <T as Config>::Oracle;
	pub type OptionConfigOf<T> = OptionConfig<AssetIdOf<T>, BalanceOf<T>, MomentOf<T>>;
	pub type OptionIdOf<T> = AssetIdOf<T>;
	pub type VaultIdOf<T> = <T as Config>::VaultId;
	pub type VaultOf<T> = <T as Config>::Vault;
	pub type VaultConfigOf<T> = VaultConfig<AccountIdOf<T>, AssetIdOf<T>>;

	// ----------------------------------------------------------------------------------------------------
	//		Storage
	// ----------------------------------------------------------------------------------------------------
	/// Maps asset_id to its vault_id
	#[pallet::storage]
	#[pallet::getter(fn asset_id_to_vault_id)]
	pub type AssetToVault<T: Config> = StorageMap<_, Blake2_128Concat, AssetIdOf<T>, VaultIdOf<T>>;

	/// Maps option_id to the option struct
	#[pallet::storage]
	#[pallet::getter(fn option_id_to_option)]
	pub type OptionIdToOption<T: Config> =
		StorageMap<_, Blake2_128Concat, OptionIdOf<T>, OptionToken<T>>;

	/// Maps option's hash with the option_id. Used to check if option exists and basically
	/// all the other searching usecases.
	#[pallet::storage]
	#[pallet::getter(fn options_hash)]
	pub type OptionHashToOptionId<T: Config> = StorageMap<_, Blake2_128Concat, H256, OptionIdOf<T>>;

	/// Maps account_id and option_id to the user's provided collateral
	#[pallet::storage]
	#[pallet::getter(fn sellers)]
	pub type Sellers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		OptionIdOf<T>,
		Blake2_128Concat,
		AccountIdOf<T>,
		SellerPosition<T>,
		OptionQuery,
	>;

	/// Maps timestamp and option_id to its currently active window_type.
	/// Scheduler is a timestamp-ordered list
	#[pallet::storage]
	pub(crate) type Scheduler<T: Config> =
		StorageDoubleMap<_, Identity, Swapped<MomentOf<T>>, Identity, OptionIdOf<T>, WindowType>;

	// ----------------------------------------------------------------------------------------------------
	//		Events
	// ----------------------------------------------------------------------------------------------------
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatedAssetVault {
			vault_id: VaultIdOf<T>,
			asset_id: AssetIdOf<T>,
		},
		CreatedOption {
			option_id: OptionIdOf<T>,
			option_config: OptionConfigOf<T>,
		},
		SellOption {
			seller: AccountIdOf<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		},
		DeleteSellOption {
			seller: AccountIdOf<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		},
		BuyOption {
			buyer: AccountIdOf<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		},
		OptionDepositStart {
			option_id: OptionIdOf<T>,
		},
		OptionPurchaseStart {
			option_id: OptionIdOf<T>,
		},
		OptionExerciseStart {
			option_id: OptionIdOf<T>,
		},
		OptionWithdrawStart {
			option_id: OptionIdOf<T>,
		},
		OptionEnd {
			option_id: OptionIdOf<T>,
		},
	}

	// ----------------------------------------------------------------------------------------------------
	//		Errors
	// ----------------------------------------------------------------------------------------------------
	#[pallet::error]
	pub enum Error<T> {
		UnexpectedError,

		// Asset vault errors
		AssetIsNotSupported,
		AssetVaultDoesNotExists,
		AssetVaultAlreadyExists,

		// Create option errors
		OptionIdDoesNotExists,
		OptionIdAlreadyExists,
		OptionAssetVaultsDoNotExist,
		OptionAttributesAreInvalid,

		// Sell option errors
		UserHasNotEnoughFundsToDeposit,
		VaultDepositNotAllowed,

		// Withdraw deposited collateral errors
		UserDoesNotHaveSellerPosition,
		UserDoesNotHaveEnoughCollateralDeposited,
		VaultWithdrawNotAllowed,

		// Epoch errors
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
		fn on_idle(_n: T::BlockNumber, remaining_weight: Weight) -> Weight {
			let mut used_weight = 0;
			let now = T::Time::now();

			while let Some((moment_swapped, option_id, moment_type)) = <Scheduler<T>>::iter().next()
			{
				used_weight = used_weight.saturating_add(T::DbWeight::get().reads(1));
				let moment = moment_swapped.into_value();

				if now < moment {
					break;
				}

				<Scheduler<T>>::remove(moment_swapped, &option_id);

				used_weight = used_weight
					.saturating_add(T::DbWeight::get().writes(1))
					.saturating_add(Self::option_state_change(option_id, moment_type));

				if used_weight >= remaining_weight {
					break;
				}
			}
			used_weight.min(remaining_weight)
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
		/// Create vault for a particular asset to deposit collateral
		#[pallet::weight(<T as Config>::WeightInfo::create_asset_vault())]
		pub fn create_asset_vault(
			origin: OriginFor<T>,
			vault_config: VaultConfigOf<T>,
		) -> DispatchResult {
			// Check if it's protocol to call the exstrinsic (TODO)
			let _from = ensure_signed(origin)?;

			<Self as TokenizedOptions>::create_asset_vault(vault_config.clone())?;

			Ok(().into())
		}

		/// Create an option to be listed for sale
		#[pallet::weight(<T as Config>::WeightInfo::create_option())]
		pub fn create_option(
			origin: OriginFor<T>,
			option_config: OptionConfigOf<T>,
		) -> DispatchResult {
			// Check if it's protocol to call the exstrinsic (TODO)
			let _from = ensure_signed(origin)?;

			<Self as TokenizedOptions>::create_option(option_config.clone())?;

			Ok(().into())
		}

		/// Sell an option providing collateral
		#[pallet::weight(<T as Config>::WeightInfo::sell_option())]
		pub fn sell_option(
			origin: OriginFor<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			<Self as TokenizedOptions>::sell_option(&from, option_amount, option_id)?;

			Ok(().into())
		}

		/// Withdraw collateral provided when selling an option
		#[pallet::weight(<T as Config>::WeightInfo::delete_sell_option())]
		pub fn delete_sell_option(
			origin: OriginFor<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			<Self as TokenizedOptions>::delete_sell_option(&from, option_amount, option_id)?;

			Ok(().into())
		}

		/// Buy an option paying premium
		#[pallet::weight(<T as Config>::WeightInfo::buy_option())]
		pub fn buy_option(
			origin: OriginFor<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			<Self as TokenizedOptions>::buy_option(&from, option_amount, option_id)?;

			Ok(().into())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		TokenizedOptions Trait
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> TokenizedOptions for Pallet<T> {
		type AccountId = AccountIdOf<T>;
		type OptionId = OptionIdOf<T>;
		type Balance = BalanceOf<T>;
		type VaultId = VaultIdOf<T>;
		type OptionConfig = OptionConfigOf<T>;
		type VaultConfig = VaultConfigOf<T>;

		/// Create vault for a particular asset to deposit collateral
		fn create_asset_vault(
			vault_config: Self::VaultConfig,
		) -> Result<Self::VaultId, DispatchError> {
			match Validated::new(vault_config) {
				Ok(validated_vault_config) => Self::do_create_asset_vault(validated_vault_config),
				Err(error) => match error {
					"ValidateVaultDoesNotExist" => {
						Err(DispatchError::from(Error::<T>::AssetVaultAlreadyExists))
					},
					"ValidateAssetIsSupported" => {
						Err(DispatchError::from(Error::<T>::AssetIsNotSupported))
					},
					_ => Err(DispatchError::from(Error::<T>::UnexpectedError)),
				},
			}
		}

		/// Create an option to be listed for sale
		fn create_option(
			option_config: Self::OptionConfig,
		) -> Result<Self::OptionId, DispatchError> {
			match Validated::new(option_config) {
				Ok(validated_option_config) => Self::do_create_option(validated_option_config),
				Err(error) => match error {
					"ValidateOptionDoesNotExist" => {
						Err(DispatchError::from(Error::<T>::OptionIdAlreadyExists))
					},
					"ValidateOptionAssetVaultsExist" => {
						Err(DispatchError::from(Error::<T>::OptionAssetVaultsDoNotExist))
					},
					"ValidateOptionAttributes" => {
						Err(DispatchError::from(Error::<T>::OptionAttributesAreInvalid))
					},
					_ => Err(DispatchError::from(Error::<T>::UnexpectedError)),
				},
			}
		}

		/// Sell an option providing collateral
		fn sell_option(
			from: &Self::AccountId,
			option_amount: Self::Balance,
			option_id: Self::OptionId,
		) -> Result<(), DispatchError> {
			ensure!(
				OptionIdToOption::<T>::contains_key(option_id),
				Error::<T>::OptionIdDoesNotExists
			);

			Self::do_sell_option(&from, option_amount, option_id)?;

			Ok(())
		}

		/// Sell an option providing collateral
		fn delete_sell_option(
			from: &Self::AccountId,
			option_amount: Self::Balance,
			option_id: Self::OptionId,
		) -> Result<(), DispatchError> {
			ensure!(
				OptionIdToOption::<T>::contains_key(option_id),
				Error::<T>::OptionIdDoesNotExists
			);

			Self::do_delete_sell_option(&from, option_amount, option_id)?;

			Ok(())
		}

		/// Buy an option paying premium
		fn buy_option(
			from: &Self::AccountId,
			option_amount: Self::Balance,
			option_id: Self::OptionId,
		) -> Result<(), DispatchError> {
			ensure!(
				OptionIdToOption::<T>::contains_key(option_id),
				Error::<T>::OptionIdDoesNotExists
			);

			Self::do_buy_option(&from, option_amount, option_id)?;

			Ok(())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Internal Pallet Functions
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> Pallet<T> {
		#[transactional]
		fn do_create_asset_vault(
			config: Validated<
				VaultConfigOf<T>,
				(ValidateVaultDoesNotExist<T>, ValidateAssetIsSupported<T>),
			>,
		) -> Result<VaultIdOf<T>, DispatchError> {
			// Get pallet account for the asset
			let account_id = Self::account_id(config.asset_id);

			// Create new vault for the asset
			let asset_vault_id: T::VaultId = T::Vault::create(
				Duration::Existential,
				VaultConfig {
					asset_id: config.asset_id,
					manager: account_id,
					reserved: config.reserved,
					strategies: config.strategies.clone(),
				},
			)?;

			// Add asset to the corresponding asset vault
			AssetToVault::<T>::insert(config.asset_id, asset_vault_id);

			Self::deposit_event(Event::CreatedAssetVault {
				vault_id: asset_vault_id,
				asset_id: config.asset_id,
			});

			Ok(asset_vault_id)
		}

		#[transactional]
		fn do_create_option(
			option_config: Validated<
				OptionConfigOf<T>,
				(
					ValidateOptionDoesNotExist<T>,
					ValidateOptionAssetVaultsExist<T>,
					ValidateOptionAttributes<T>,
				),
			>,
		) -> Result<OptionIdOf<T>, DispatchError> {
			// Generate new option_id for the option token
			let option_id = T::CurrencyFactory::create(RangeId::LP_TOKENS)?;

			let option = OptionToken {
				base_asset_id: option_config.base_asset_id,
				quote_asset_id: option_config.quote_asset_id,
				base_asset_strike_price: option_config.base_asset_strike_price,
				quote_asset_strike_price: option_config.quote_asset_strike_price,
				option_type: option_config.option_type,
				exercise_type: option_config.exercise_type,
				expiring_date: option_config.expiring_date,
				base_asset_amount_per_option: option_config.base_asset_amount_per_option,
				quote_asset_amount_per_option: option_config.quote_asset_amount_per_option,
				total_issuance_seller: option_config.total_issuance_seller,
				total_issuance_buyer: option_config.total_issuance_buyer,
				epoch: option_config.epoch,
			};

			let option_hash = option.generate_id();

			// Add option_id to corresponding option
			OptionHashToOptionId::<T>::insert(option_hash, option_id);
			OptionIdToOption::<T>::insert(option_id, option);
			Self::schedule_option(option_config.epoch, option_id);

			Self::deposit_event(Event::CreatedOption {
				option_id,
				option_config: option_config.value(),
			});

			Ok(option_id)
		}

		#[transactional]
		fn do_sell_option(
			from: &AccountIdOf<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		) -> Result<(), DispatchError> {
			let option =
				Self::option_id_to_option(option_id).ok_or(Error::<T>::OptionIdDoesNotExists)?;

			// Check if we are in deposit window
			ensure!(
				option
					.epoch
					.window_type(T::Time::now())
					.ok_or(Error::<T>::NotIntoDepositWindow)?
					== WindowType::Deposit,
				Error::<T>::NotIntoDepositWindow
			);

			// Different behaviors based on Call or Put option
			let mut asset_id = option.base_asset_id;
			let mut asset_amount = option
				.base_asset_amount_per_option
				.checked_mul(&option_amount)
				.ok_or(ArithmeticError::Overflow)?;

			if option.option_type == OptionType::Put {
				asset_id = option.quote_asset_id;
				asset_amount = option
					.base_asset_strike_price
					.checked_mul(&option_amount)
					.ok_or(ArithmeticError::Overflow)?;
			};

			// Get vault_id and protocol account for depositing collateral
			let vault_id =
				Self::asset_id_to_vault_id(asset_id).ok_or(Error::<T>::AssetVaultDoesNotExists)?;

			let protocol_account = Self::account_id(asset_id);

			// Calculate the amount of shares the user should get and make checks
			let shares_amount = T::Vault::calculate_lp_tokens_to_mint(&vault_id, asset_amount)?;

			// Update position or create position
			if Sellers::<T>::contains_key(option_id, from) {
				Sellers::<T>::try_mutate(
					option_id,
					from,
					|position| -> Result<(), DispatchError> {
						match position {
							Some(position) => {
								// Add option amount to position
								let new_option_amount = position
									.option_amount
									.checked_add(&option_amount)
									.ok_or(ArithmeticError::Overflow)?;

								// Add shares amount to position
								let new_shares_amount = position
									.shares_amount
									.checked_add(&shares_amount)
									.ok_or(ArithmeticError::Overflow)?;

								position.option_amount = new_option_amount;
								position.shares_amount = new_shares_amount;

								// Transfer collateral to protocol account
								<T as Config>::MultiCurrency::transfer(
									asset_id,
									&from,
									&protocol_account,
									asset_amount,
									true,
								)
								.map_err(|_| Error::<T>::UserHasNotEnoughFundsToDeposit)?;

								// Protocol account deposits into the vault and receives
								// shares_amount
								T::Vault::deposit(&vault_id, &protocol_account, asset_amount)
									.map_err(|_| Error::<T>::VaultDepositNotAllowed)?;

								Self::deposit_event(Event::SellOption {
									seller: from.clone(),
									option_amount,
									option_id,
								});

								Ok(())
							},
							None => Err(DispatchError::from(Error::<T>::UnexpectedError)),
						}
					},
				)?;
			} else {
				let position = SellerPosition { option_amount, shares_amount };
				Sellers::<T>::insert(option_id, from, position);

				// Transfer collateral to protocol account
				<T as Config>::MultiCurrency::transfer(
					asset_id,
					&from,
					&protocol_account,
					asset_amount,
					true,
				)
				.map_err(|_| Error::<T>::UserHasNotEnoughFundsToDeposit)?;

				// Protocol account deposits into the vault and keep shares_amount
				T::Vault::deposit(&vault_id, &protocol_account, asset_amount)
					.map_err(|_| Error::<T>::VaultDepositNotAllowed)?;

				Self::deposit_event(Event::SellOption {
					seller: from.clone(),
					option_amount,
					option_id,
				});
			}

			OptionIdToOption::<T>::try_mutate(option_id, |option| {
				match option {
					Some(option) => {
						// Add option amount to total issuance
						let new_total_issuance_seller = option
							.total_issuance_seller
							.checked_add(&option_amount)
							.ok_or(ArithmeticError::Overflow)?;

						option.total_issuance_seller = new_total_issuance_seller;

						Ok(())
					},
					None => Err(DispatchError::from(Error::<T>::UnexpectedError)),
				}
			})
		}

		#[transactional]
		fn do_delete_sell_option(
			from: &AccountIdOf<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		) -> Result<(), DispatchError> {
			let option =
				Self::option_id_to_option(option_id).ok_or(Error::<T>::OptionIdDoesNotExists)?;

			// Check if we are in deposit window
			ensure!(
				option
					.epoch
					.window_type(T::Time::now())
					.ok_or(Error::<T>::NotIntoDepositWindow)?
					== WindowType::Deposit,
				Error::<T>::NotIntoDepositWindow
			);

			// Check if user has deposited any collateral before and retrieve position
			let seller_position = Sellers::<T>::try_get(option_id, from)
				.map_err(|_| Error::<T>::UserDoesNotHaveSellerPosition)?;

			// Different behaviors based on Call or Put option
			let mut asset_id = option.base_asset_id;
			let mut asset_amount = option
				.base_asset_amount_per_option
				.checked_mul(&option_amount)
				.ok_or(ArithmeticError::Overflow)?;

			if option.option_type == OptionType::Put {
				asset_id = option.quote_asset_id;
				asset_amount = option
					.base_asset_strike_price
					.checked_mul(&option_amount)
					.ok_or(ArithmeticError::Overflow)?;
			};

			// Get vault_id for withdrawing collateral and make checks
			let protocol_account = Self::account_id(asset_id);

			let vault_id =
				Self::asset_id_to_vault_id(asset_id).ok_or(Error::<T>::AssetVaultDoesNotExists)?;

			let shares_amount = Self::calculate_shares_to_burn(option_amount, &seller_position)?;

			// Correct logic checks
			ensure!(
				asset_amount == T::Vault::lp_share_value(&vault_id, shares_amount)?
					&& asset_amount
						<= T::Vault::lp_share_value(&vault_id, seller_position.shares_amount)?
					&& option_amount <= seller_position.option_amount,
				Error::<T>::UserDoesNotHaveEnoughCollateralDeposited
			);

			// Update position or delete position
			if shares_amount != seller_position.shares_amount {
				Sellers::<T>::try_mutate(
					option_id,
					from,
					|position| -> Result<(), DispatchError> {
						match position {
							Some(position) => {
								// Subtract option amount to position
								let new_option_amount = position
									.option_amount
									.checked_sub(&option_amount)
									.ok_or(ArithmeticError::Overflow)?;

								// Subtract shares amount to position
								let new_shares_amount = position
									.shares_amount
									.checked_sub(&shares_amount)
									.ok_or(ArithmeticError::Overflow)?;

								position.option_amount = new_option_amount;
								position.shares_amount = new_shares_amount;

								// Protocol account withdraw from the vault and burn
								// shares_amount
								T::Vault::withdraw(&vault_id, &protocol_account, shares_amount)
									.map_err(|_| Error::<T>::VaultWithdrawNotAllowed)?;

								// Transfer collateral to user account
								<T as Config>::MultiCurrency::transfer(
									asset_id,
									&protocol_account,
									&from,
									asset_amount,
									true,
								)?;

								Self::deposit_event(Event::DeleteSellOption {
									seller: from.clone(),
									option_amount,
									option_id,
								});

								Ok(())
							},
							None => Err(DispatchError::from(Error::<T>::UnexpectedError)),
						}
					},
				)?;
			} else {
				Sellers::<T>::remove(option_id, from);

				// Protocol account withdraw from the vault and burn shares_amount
				T::Vault::withdraw(&vault_id, &protocol_account, shares_amount)
					.map_err(|_| Error::<T>::VaultWithdrawNotAllowed)?;

				// Transfer collateral to user account
				<T as Config>::MultiCurrency::transfer(
					asset_id,
					&protocol_account,
					&from,
					asset_amount,
					true,
				)?;

				Self::deposit_event(Event::DeleteSellOption {
					seller: from.clone(),
					option_amount,
					option_id,
				});
			}

			OptionIdToOption::<T>::try_mutate(option_id, |option| {
				match option {
					Some(option) => {
						// Subtract option amount to total issuance
						let new_total_issuance_seller = option
							.total_issuance_seller
							.checked_sub(&option_amount)
							.ok_or(ArithmeticError::Overflow)?;

						option.total_issuance_seller = new_total_issuance_seller;

						Ok(())
					},
					None => Err(DispatchError::from(Error::<T>::UnexpectedError)),
				}
			})
		}

		#[transactional]
		fn do_buy_option(
			from: &AccountIdOf<T>,
			option_amount: BalanceOf<T>,
			option_id: OptionIdOf<T>,
		) -> Result<(), DispatchError> {
			Self::deposit_event(Event::BuyOption { buyer: from.clone(), option_amount, option_id });

			Ok(())
		}

		// ----------------------------------------------------------------------------------------------------
		//		Helper Functions
		// ----------------------------------------------------------------------------------------------------
		/// Protocol account for a particular asset
		pub fn account_id(asset: AssetIdOf<T>) -> AccountIdOf<T> {
			T::PalletId::get().into_sub_account(asset)
		}

		/// Calculate the hash of an option providing the required attributes
		pub fn generate_id(
			base_asset_id: AssetIdOf<T>,
			quote_asset_id: AssetIdOf<T>,
			base_asset_strike_price: BalanceOf<T>,
			quote_asset_strike_price: BalanceOf<T>,
			option_type: OptionType,
			expiring_date: MomentOf<T>,
			exercise_type: ExerciseType,
		) -> H256 {
			BlakeTwo256::hash_of(&(
				base_asset_id,
				quote_asset_id,
				base_asset_strike_price,
				quote_asset_strike_price,
				option_type,
				expiring_date,
				exercise_type,
			))
		}

		pub fn calculate_shares_to_burn(
			option_amount: BalanceOf<T>,
			seller_position: &SellerPosition<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			let a =
				<T::Convert as Convert<BalanceOf<T>, u128>>::convert(seller_position.shares_amount);
			let b = <T::Convert as Convert<BalanceOf<T>, u128>>::convert(option_amount);
			let c =
				<T::Convert as Convert<BalanceOf<T>, u128>>::convert(seller_position.option_amount);

			let shares_amount =
				multiply_by_rational(a, b, c).map_err(|_| ArithmeticError::Overflow)?;

			let shares_amount = <T::Convert as Convert<u128, T::Balance>>::convert(shares_amount);
			Ok(shares_amount)
		}

		fn schedule_option(epoch: Epoch<MomentOf<T>>, option_id: OptionIdOf<T>) {
			<Scheduler<T>>::insert(Swapped::from(epoch.deposit), option_id, WindowType::Deposit);
			<Scheduler<T>>::insert(Swapped::from(epoch.purchase), option_id, WindowType::Purchase);
			<Scheduler<T>>::insert(Swapped::from(epoch.exercise), option_id, WindowType::Exercise);
			<Scheduler<T>>::insert(Swapped::from(epoch.withdraw), option_id, WindowType::Withdraw);
			<Scheduler<T>>::insert(Swapped::from(epoch.end), option_id, WindowType::End);
		}

		fn option_state_change(option_id: OptionIdOf<T>, moment_type: WindowType) -> Weight {
			match moment_type {
				WindowType::Deposit => Self::option_deposit_start(option_id),
				WindowType::Purchase => Self::option_purchase_start(option_id),
				WindowType::Exercise => Self::option_exercise_start(option_id),
				WindowType::Withdraw => Self::option_withdraw_start(option_id),
				WindowType::End => Self::option_end(option_id),
			}
		}

		fn option_deposit_start(option_id: OptionIdOf<T>) -> Weight {
			Self::deposit_event(Event::OptionDepositStart { option_id });
			0
		}

		fn option_purchase_start(option_id: OptionIdOf<T>) -> Weight {
			Self::deposit_event(Event::OptionPurchaseStart { option_id });
			0
		}

		fn option_exercise_start(option_id: OptionIdOf<T>) -> Weight {
			Self::deposit_event(Event::OptionExerciseStart { option_id });
			0
		}

		fn option_withdraw_start(option_id: OptionIdOf<T>) -> Weight {
			Self::deposit_event(Event::OptionWithdrawStart { option_id });
			0
		}

		fn option_end(option_id: OptionIdOf<T>) -> Weight {
			Self::deposit_event(Event::OptionEnd { option_id });
			0
		}
	}
}

// ----------------------------------------------------------------------------------------------------
//		Unit Tests
// ----------------------------------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {}
