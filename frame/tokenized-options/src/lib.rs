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
	use composable_traits::tokenized_options::{ExerciseType, OptionType, TokenizedOptions};
	use sp_runtime::DispatchError;

	use composable_traits::vault::{CapabilityVault, Deposit as Duration, Vault, VaultConfig};
	use frame_support::pallet_prelude::*;
	use frame_support::{transactional, PalletId};
	use frame_system::ensure_signed;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{
		AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, CheckedMul, CheckedSub, Zero,
	};
	use sp_std::fmt::Debug;

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

		type WeightInfo: WeightInfo;

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

		type VaultId: Clone + Codec + MaxEncodedLen + Debug + PartialEq + Default + Parameter;

		type Vault: CapabilityVault<
			AssetId = Self::AssetId,
			Balance = Self::Balance,
			AccountId = Self::AccountId,
			VaultId = Self::VaultId,
		>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	// ----------------------------------------------------------------------------------------------------
	//		Helper Pallet Types
	// ----------------------------------------------------------------------------------------------------
	type AssetIdOf<T> = <T as Config>::AssetId;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct OptionToken<T: Config> {
		pub asset_id: T::AssetId,
		pub strike_price: T::Balance,
		pub premium: T::Balance,
		pub option_type: OptionType,
		pub exercise_type: ExerciseType,
	}

	// ----------------------------------------------------------------------------------------------------
	//		Storage
	// ----------------------------------------------------------------------------------------------------
	#[pallet::storage]
	/// Supported collateral asset ids
	pub type CollateralTypes<T: Config> = StorageMap<_, Twox64Concat, AssetIdOf<T>, ()>;

	#[pallet::storage]
	#[pallet::getter(fn asset_vault)]
	/// Maps each asset to its vault
	pub type AssetVault<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, T::VaultId>;

	#[pallet::storage]
	#[pallet::getter(fn account_asset_collateral)]
	/// Maps each account to its collateral for a particular asset
	pub type AccountAssetCollateral<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Twox64Concat, T::AssetId, T::Balance>;

	// ----------------------------------------------------------------------------------------------------
	//		Events
	// ----------------------------------------------------------------------------------------------------
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Create { asset: T::AssetId },
	}

	// ----------------------------------------------------------------------------------------------------
	//		Errors
	// ----------------------------------------------------------------------------------------------------
	#[pallet::error]
	pub enum Error<T> {}

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
		pub collateral_types: Vec<AssetIdOf<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { collateral_types: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.collateral_types.iter().for_each(|asset| {
				CollateralTypes::<T>::insert(asset, ());
			})
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Extrinsics
	// ----------------------------------------------------------------------------------------------------
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[transactional]
		#[pallet::weight(<T as Config>::WeightInfo::create())]
		// #[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>, asset: T::AssetId) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;

			Self::do_create(from, asset)?;

			Self::deposit_event(Event::Create { asset });

			Ok(().into())
		}
	}

	// ----------------------------------------------------------------------------------------------------
	//		Options Trait
	// ----------------------------------------------------------------------------------------------------
	// impl<T: Config> TokenizedOptions for Pallet<T> {}

	// ----------------------------------------------------------------------------------------------------
	//		Internal Functions
	// ----------------------------------------------------------------------------------------------------
	impl<T: Config> Pallet<T> {
		fn account_id(asset: &T::AssetId) -> T::AccountId {
			T::PalletId::get().into_sub_account(asset)
		}

		fn do_create(_issuer: T::AccountId, asset: T::AssetId) -> Result<(), DispatchError> {
			AssetVault::<T>::insert(asset, T::VaultId::default());

			Ok(())
		}
	}
}

// ----------------------------------------------------------------------------------------------------
//		Unit Tests
// ----------------------------------------------------------------------------------------------------
#[cfg(test)]
mod unit_tests {}
