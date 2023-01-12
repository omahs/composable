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
#![deny(clippy::unseparated_literal_suffix, unused_imports, dead_code)]
#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
pub use pallet::*;
use scale_info::TypeInfo;

#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarking;
#[cfg(test)]
mod runtime;

#[cfg(test)]
mod tests;

mod prelude;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use crate::prelude::*;
	pub use crate::weights::WeightInfo;
	use codec::FullCodec;
	use composable_traits::{
		assets::Asset,
		currency::{BalanceLike, CurrencyFactory, Exponent, ForeignByNative, RangeId},
		xcm::assets::{RemoteAssetRegistryInspect, RemoteAssetRegistryMutate},
	};
	use cumulus_primitives_core::ParaId;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::EnsureOrigin, BoundedVec,
		Twox128,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_std::{fmt::Debug, str, vec::Vec};

	/// The module configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Identifier for the class of local asset.
		type LocalAssetId: FullCodec
			+ Eq
			+ PartialEq
			+ Copy
			+ MaybeSerializeDeserialize
			+ From<u128>
			+ Into<u128>
			+ Debug
			+ Default
			+ Ord
			+ TypeInfo
			+ MaxEncodedLen;

		/// Identifier for the class of foreign asset.
		type ForeignAssetId: FullCodec
			+ Eq
			+ PartialEq
			+ MaybeSerializeDeserialize
			+ Debug
			+ Clone
			+ Default
			+ TypeInfo
			+ MaxEncodedLen;

		/// The origin which may set local and foreign admins.
		type UpdateAssetRegistryOrigin: EnsureOrigin<Self::Origin>;
		/// really can be governance of this chain or remote parachain origin
		type ParachainOrGovernanceOrigin: EnsureOrigin<Self::Origin>;
		type WeightInfo: WeightInfo;
		type Balance: BalanceLike;
		type CurrencyFactory: CurrencyFactory<AssetId = Self::LocalAssetId, Balance = Self::Balance>;
		// TODO(RFC-0013): Update Assets Registry - Pallet Configuration
		// | Provide configuration item for asset creation to assets-registry
		// type CreateLocalAssets: fungibles::Create<Self::AccountId>;
		// type CreateForeignAssetsAssets: fungibles::Create<Self::AccountId>;
		// | Provide configuration item for max length of ticker-symbols
		#[pallet::constant]
		type AssetSymbolMaxChars: Get<u32>;
		#[pallet::constant]
		type AssetNameMaxChars: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Mapping local asset to foreign asset.
	#[pallet::storage]
	#[pallet::getter(fn from_local_asset)]
	pub type LocalToForeign<T: Config> =
		StorageMap<_, Twox128, T::LocalAssetId, T::ForeignAssetId, OptionQuery>;

	/// Mapping foreign asset to local asset.
	#[pallet::storage]
	#[pallet::getter(fn from_foreign_asset)]
	pub type ForeignToLocal<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAssetId, T::LocalAssetId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn minimal_amount)]
	pub type MinFeeAmounts<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ParaId,
		Blake2_128Concat,
		T::ForeignAssetId,
		T::Balance,
		OptionQuery,
	>;

	/// How much of asset amount is needed to pay for one unit of native token.
	#[pallet::storage]
	#[pallet::getter(fn asset_ratio)]
	pub type AssetRatio<T: Config> = StorageMap<_, Twox128, T::LocalAssetId, Rational, OptionQuery>;

	/// Metadata of an asset
	#[pallet::storage]
	#[pallet::getter(fn decimals)]
	pub type Metadata<T: Config> = StorageMap<
		_,
		Twox128,
		T::LocalAssetId,
		crate::AssetMetadata<
			BoundedVec<u8, T::AssetNameMaxChars>,
			BoundedVec<u8, T::AssetSymbolMaxChars>,
		>,
		OptionQuery,
	>;

	// TODO(RFC-0013): Update Assets Registry - Pallet Storage
	// | Add a nonce storage item that will be used for generating foreign asset IDs
	// #[pallet::storage]
	// #[pallet::getter(fn foreign_asset_id_nonce)]
	// #[allow(clippy::disallowed_types)] // nonce, ValueQuery is OK
	// pub type VestingScheduleNonce<T: Config> =
	// 	StorageValue<_, u64, ValueQuery, Nonce<ZeroInit, SafeIncrement>>;
	// | Add storage item for Asset ticker-symbol
	// #[pallet::storage]
	// #[pallet::getter(fn asset_ticker_symbol)]
	// pub type AssetTickerSymbol<T: Config> = StorageMap<
	// 	_,
	// 	Twox128,
	// 	T::LocalAssetId,
	// 	BoundedVec<u8, T::TickerSymbolMaxChars>,
	// 	OptionQuery,
	// >;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config>(sp_std::marker::PhantomData<T>);

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self(<_>::default())
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T>
	where
		composable_traits::xcm::assets::XcmAssetLocation:
			codec::EncodeLike<<T as Config>::ForeignAssetId>,
	{
		fn build(&self) {}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetRegistered {
			asset_id: T::LocalAssetId,
			location: T::ForeignAssetId,
			decimals: Option<Exponent>,
		},
		AssetUpdated {
			asset_id: T::LocalAssetId,
			location: T::ForeignAssetId,
			decimals: Option<Exponent>,
		},
		MinFeeUpdated {
			target_parachain_id: ParaId,
			foreign_asset_id: T::ForeignAssetId,
			amount: Option<T::Balance>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		AssetNotFound,
		ForeignAssetAlreadyRegistered,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates asset using `CurrencyFactory`.
		/// Raises `AssetRegistered` event
		///
		/// Sets only required fields by `CurrencyFactory`, to upsert metadata use referenced
		/// pallet.
		///
		/// # Parameters:
		///
		/// `ratio` -  
		/// Allows `bring you own gas` fees.
		/// Set to `None` to prevent payment in this asset, only transferring.
		/// Setting to some will NOT start minting tokens with specified ratio.
		///
		/// ```python
		///  ratio = foreign_token / native_token
		///  amount_of_foreign_asset = amount_of_native_asset * ratio
		/// ```
		///
		/// `decimals` - `human` number of decimals
		///
		/// `ed` - same meaning as in for foreign asset account (if None, then asset is not
		/// sufficient)
		#[pallet::weight(<T as Config>::WeightInfo::register_asset())]
		// TODO(RFC-0013): Update Assets Registry - Pallet Functions
		// | Update `register_*_asset` routes
		// | Split this into `register_local_asset` and `register_foreign_asset` call functions
		// | Add backing function that is generic over each of them
		pub fn register_asset(
			origin: OriginFor<T>,
			location: T::ForeignAssetId,
			ratio: Rational,
			decimals: Option<Exponent>,
		) -> DispatchResultWithPostInfo {
			T::UpdateAssetRegistryOrigin::ensure_origin(origin)?;
			ensure!(
				!ForeignToLocal::<T>::contains_key(&location),
				Error::<T>::ForeignAssetAlreadyRegistered
			);
			let asset_id = T::CurrencyFactory::create(RangeId::FOREIGN_ASSETS)?;
			Self::set_reserve_location(asset_id, location.clone(), ratio)?;
			Self::deposit_event(Event::<T>::AssetRegistered { asset_id, location, decimals });
			Ok(().into())
		}

		/// Given well existing asset, update its remote information.
		/// Use with caution as it allow reroute assets location.
		/// See `register_asset` for parameters meaning.
		#[pallet::weight(<T as Config>::WeightInfo::update_asset())]
		// TODO(RFC-0013): Update Assets Registry - Pallet Functions
		// | Update `update_*_asset` routes
		// | Split this into `update_local_asset` and `update_foreign_asset` call functions
		// | Add backing function that is generic over each of them
		pub fn update_asset(
			origin: OriginFor<T>,
			asset_id: T::LocalAssetId,
			location: T::ForeignAssetId,
			ratio: Rational,
			decimals: Option<Exponent>,
		) -> DispatchResultWithPostInfo {
			T::UpdateAssetRegistryOrigin::ensure_origin(origin)?;
			Self::set_reserve_location(asset_id, location.clone(), ratio)?;
			Self::deposit_event(Event::<T>::AssetUpdated { asset_id, location, decimals });
			Ok(().into())
		}

		/// Minimal amount of `foreign_asset_id` required to send message to other network.
		/// Target network may or may not accept payment `amount`.
		/// Assumed this is maintained up to date by technical team.
		/// Mostly UI hint and fail fast solution.
		/// Messages sending smaller fee will not be sent.
		/// In theory can be updated by parachain sovereign account too.
		/// If None, than it is well known cannot pay with that asset on target_parachain_id.
		/// If Some(0), than price can be anything greater or equal to zero.
		/// If Some(MAX), than actually it forbids transfers.
		#[pallet::weight(<T as Config>::WeightInfo::set_min_fee())]
		pub fn set_min_fee(
			origin: OriginFor<T>,
			target_parachain_id: ParaId,
			foreign_asset_id: T::ForeignAssetId,
			amount: Option<T::Balance>,
		) -> DispatchResultWithPostInfo {
			T::ParachainOrGovernanceOrigin::ensure_origin(origin)?;
			// TODO: in case it is set to parachain, check that chain can target only its origin
			MinFeeAmounts::<T>::mutate_exists(target_parachain_id, foreign_asset_id.clone(), |x| {
				*x = amount
			});
			Self::deposit_event(Event::<T>::MinFeeUpdated {
				target_parachain_id,
				foreign_asset_id,
				amount,
			});
			Ok(().into())
		}
	}

	impl<T: Config> RemoteAssetRegistryMutate for Pallet<T> {
		type AssetId = T::LocalAssetId;

		type AssetNativeLocation = T::ForeignAssetId;

		type Balance = T::Balance;

		fn set_reserve_location(
			asset_id: Self::AssetId,
			location: Self::AssetNativeLocation,
			ratio: Rational,
		) -> DispatchResult {
			ForeignToLocal::<T>::insert(&location, asset_id);
			LocalToForeign::<T>::insert(asset_id, location);
			AssetRatio::<T>::mutate_exists(asset_id, |x| *x = Some(ratio));
			Ok(())
		}

		fn update_ratio(
			location: Self::AssetNativeLocation,
			ratio: Option<Rational>,
		) -> DispatchResult {
			let asset_id =
				ForeignToLocal::<T>::try_get(location).map_err(|_| Error::<T>::AssetNotFound)?;
			AssetRatio::<T>::mutate_exists(asset_id, |x| *x = ratio);
			Ok(())
		}
	}

	impl<T: Config> RemoteAssetRegistryInspect for Pallet<T> {
		type AssetId = T::LocalAssetId;
		type AssetNativeLocation = T::ForeignAssetId;
		type Balance = T::Balance;

		fn asset_to_remote(asset_id: Self::AssetId) -> Option<Self::AssetNativeLocation> {
			LocalToForeign::<T>::get(asset_id)
		}

		fn location_to_asset(location: Self::AssetNativeLocation) -> Option<Self::AssetId> {
			ForeignToLocal::<T>::get(location)
		}

		fn min_xcm_fee(
			parachain_id: ParaId,
			remote_asset_id: Self::AssetNativeLocation,
		) -> Option<Self::Balance> {
			<MinFeeAmounts<T>>::get(parachain_id, remote_asset_id)
		}

		fn get_foreign_assets_list() -> Vec<Asset<T::Balance, Self::AssetNativeLocation>> {
			ForeignToLocal::<T>::iter()
				.map(|(_, asset_id)| {
					let foreign_id = LocalToForeign::<T>::get(asset_id)
						.expect("Must exist, as it does in ForeignToLocal");
					let decimals =
						<Pallet<T> as crate::InspectRegistryMetadata>::decimals(&asset_id)
							.unwrap_or(12);
					let ratio = AssetRatio::<T>::get(asset_id);

					Asset {
						name: None,
						id: asset_id.into(),
						decimals,
						ratio,
						foreign_id: Some(foreign_id),
						existential_deposit: T::Balance::default(),
					}
				})
				.collect::<Vec<_>>()
		}
	}

	impl<T: Config> AssetRatioInspect for Pallet<T> {
		type AssetId = T::LocalAssetId;
		fn get_ratio(asset_id: Self::AssetId) -> Option<ForeignByNative> {
			AssetRatio::<T>::get(asset_id).map(Into::into)
		}
	}

	impl<T: Config> crate::InspectRegistryMetadata for Pallet<T> {
		type AssetId = T::LocalAssetId;

		/// Return the name of an asset.
		fn name(asset: &Self::AssetId) -> Option<Vec<u8>> {
			Metadata::<T>::get(asset).map(|metadata| metadata.name.to_vec())
		}

		/// Return the symbol of an asset.
		fn symbol(asset: &Self::AssetId) -> Option<Vec<u8>> {
			// Metadata::<T>::get(asset).symbol.to_vec()
			Metadata::<T>::get(asset).map(|metadata| metadata.symbol.to_vec())
		}

		/// Return the decimals of an asset.
		fn decimals(asset: &Self::AssetId) -> Option<u8> {
			Metadata::<T>::get(asset).map(|metadata| metadata.decimals)
		}
	}
}

/// Routing of indepent parts of the `AssetMetadata` from `pallet-assets-registry`
pub trait InspectRegistryMetadata {
	type AssetId;

	/// Return the name of an asset.
	fn name(asset: &Self::AssetId) -> Option<Vec<u8>>;
	/// Return the symbol of an asset.
	fn symbol(asset: &Self::AssetId) -> Option<Vec<u8>>;
	/// Return the decimals of an asset.
	fn decimals(asset: &Self::AssetId) -> Option<u8>;
}

/// Structure to represent basic asset metadata such as: name, symbol, decimals.
#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetMetadata<BoundedName, BoundedSymbol> {
	/// Name of the asset.
	pub name: BoundedName,
	/// Symbol of the asset.
	pub symbol: BoundedSymbol,
	/// The number of decimals this asset uses to represent one unit.
	pub decimals: u8,
}
