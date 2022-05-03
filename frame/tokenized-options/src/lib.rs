//! # Options Pallet

#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(test)]
// mod tests;

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
	use composable_traits::{swap_bytes::SwapBytes, tokenized_options::*};
	use sp_runtime::DispatchError;

	use composable_traits::vault::{CapabilityVault, Deposit as Duration, Vault, VaultConfig};
	use frame_support::pallet_prelude::*;
	use frame_support::{
		storage::bounded_vec::BoundedVec,
		traits::{
			fungible::{Inspect as NativeInspect, Transfer as NativeTransfer},
			fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
			Time,
		},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use frame_system::{ensure_root, ensure_signed};
	use sp_runtime::{
		traits::{
			AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned, CheckedAdd, CheckedMul,
			CheckedSub, Zero,
		},
		Perquintill,
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

		/// tokenized_options PalletId
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type WeightInfo: WeightInfo;

		/// Type of time moment. NB: we use swap_bytes to store this type in big-endian format
		/// and take advantage of the fact that storage keys are stored in lexical order.
		type Moment: SwapBytes + AtLeast32Bit + Parameter + Copy + MaxEncodedLen;

		/// The time provider.
		type Time: Time<Moment = Self::Moment>;

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

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct OptionToken<AssetId, Balance, Moment> {
		pub base_asset_id: AssetId,
		pub quote_asset_id: AssetId,
		pub base_asset_strike_price: Balance,
		// pub quote_asset_strike_price: Balance, // Assume stablecoin as quote asset right now, so always 1
		pub option_type: OptionType,
		pub exercise_type: ExerciseType,
		pub expiring_date: Moment,
		pub base_asset_amount_per_option: Balance,
		// pub quote_asset_amount_per_option: Balance, // Assume stablecoin as quote asset right now, so always 1
		pub total_issuance_seller: Balance,
		pub total_issuance_buyer: Balance,
		pub epoch: Epoch<Moment>,
	}

	#[derive(Clone, Copy, Encode, Decode, Debug, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
	pub enum MomentType {
		Deposit,
		Purchase,
		Exercise,
		Withdraw,
		End,
	}

	#[derive(Clone, Copy, Encode, Decode, Debug, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
	pub struct Epoch<Moment> {
		pub deposit: Moment,
		pub purchase: Moment,
		pub exercise: Moment,
		pub withdraw: Moment,
		pub end: Moment,
	}

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type AssetIdOf<T> = <T as DeFiComposableConfig>::MayBeAssetId;
	type BalanceOf<T> = <T as DeFiComposableConfig>::Balance;
	type VaultIdOf<T> = <T as Config>::VaultId;
	type MomentOf<T> = <T as Config>::Moment;
	type VaultOf<T> = <T as Config>::Vault;
	type OptionOf<T> = OptionToken<AssetIdOf<T>, BalanceOf<T>, MomentOf<T>>;

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

	#[pallet::storage]
	pub(crate) type Schedule<T: Config> =
		StorageDoubleMap<_, Identity, T::Moment, Identity, AssetIdOf<T>, MomentType>;

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
		) -> DispatchResult {
			let from = ensure_signed(_origin)?;

			let vault_id = Self::do_create_asset_vault(&_config)?;

			Self::deposit_event(Event::CreatedAssetVault { vault_id, asset_id: _config.asset_id });

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::create_option())]
		pub fn create_option(_origin: OriginFor<T>, _option: OptionOf<T>) -> DispatchResult {
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
		) -> DispatchResult {
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
		) -> DispatchResult {
			let from = ensure_signed(_origin)?;

			<Self as TokenizedOptions>::buy_option(from, _amount, _option_id)?;

			// Self::deposit_event(Event::BuyOption);

			Ok(().into())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Hooks
	// ----------------------------------------------------------------------------------------------------
	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_idle(_n: T::BlockNumber, _remaining_weight: Weight) -> Weight {
			let now = T::Time::now();
			while let Some((moment, option_id, typ)) = <Schedule<T>>::iter().next() {
				let moment = moment.swap_bytes();
				if moment > now {
					break;
				}
				<Schedule<T>>::remove(&moment, &option_id);
				match typ {
					MomentType::Deposit => Self::option_deposit_start(option_id),
					MomentType::Purchase => Self::option_purchase_start(option_id),
					MomentType::Exercise => Self::option_exercise_start(option_id),
					MomentType::Withdraw => Self::option_withdraw_start(option_id),
					MomentType::End => Self::option_end(option_id),
				}
				.unwrap();
			}
			10_000
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
			option: &Self::OptionToken,
		) -> Result<Self::AssetId, DispatchError> {
			let option_id = Self::do_create_option(&option).unwrap();

			Ok(option_id)
		}

		fn sell_option(
			from: &Self::AccountId,
			amount: Self::Balance,
			option_id: Self::AssetId,
		) -> Result<(), DispatchError> {
			ensure!(OptionIdToOption::<T>::contains_key(option_id), Error::<T>::OptionIdNotFound);

			Self::do_sell_option(&from, amount, option_id)?;

			Ok(())
		}

		fn buy_option(
			from: Self::AccountId,
			amount: Self::Balance,
			option_id: Self::AssetId,
		) -> Result<(), DispatchError> {
			ensure!(OptionIdToOption::<T>::contains_key(option_id), Error::<T>::OptionIdNotFound);

			Self::do_buy_option(from, amount, option_id)?;

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
		fn do_create_option(option: &OptionOf<T>) -> Result<AssetIdOf<T>, DispatchError> {
			// Check if option already exists (how?)

			// Generate new option_id for the option token
			let option_id = T::CurrencyFactory::create(RangeId::LP_TOKENS)?;

			// Add option_id to corresponding option
			OptionIdToOption::<T>::insert(option_id, option);

			Ok(option_id)
		}

		#[transactional]
		fn do_sell_option(
			from: &AccountIdOf<T>,
			amount: BalanceOf<T>,
			option_id: AssetIdOf<T>,
		) -> Result<(), DispatchError> {
			// Get pallet option_account
			let account_id = Self::account_id(option_id);

			let option =
				Self::option_id_to_option(option_id).ok_or(Error::<T>::OptionIdNotFound)?;

			// Do stuff with option (based on option's attributes)

			// Right now I'm simply calling this since there is no a fixed design for options yet
			// and I'm setting the test env for this function
			<T as Config>::MultiCurrency::mint_into(option_id, from, amount)?;

			Ok(())
		}

		#[transactional]
		fn do_buy_option(
			_from: AccountIdOf<T>,
			_amount: BalanceOf<T>,
			option_id: AssetIdOf<T>,
		) -> Result<(), DispatchError> {
			// Get pallet option_account
			let account_id = Self::account_id(option_id);

			let option =
				Self::option_id_to_option(option_id).ok_or(Error::<T>::OptionIdNotFound)?;

			Ok(())
		}
	}

	impl<Moment: Ord> Epoch<Moment> {
		pub fn moment_type(&self, moment: Moment) -> Option<MomentType> {
			if moment < self.deposit {
				None
			} else if moment < self.purchase {
				Some(MomentType::Deposit)
			} else if moment < self.exercise {
				Some(MomentType::Purchase)
			} else if moment < self.withdraw {
				Some(MomentType::Exercise)
			} else if moment < self.end {
				Some(MomentType::Withdraw)
			} else {
				Some(MomentType::End)
			}
		}
	}
}

// ----------------------------------------------------------------------------------------------------
//		Unit Tests
// ----------------------------------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {}
