// // mod protocol;

// // ///
// // use codec::{Decode, Encode, FullCodec, MaxEncodedLen};
// // use composable_support::collections::vec::bounded::BiBoundedVec;
// // use core::fmt::Debug;
// // use frame_support::{
// // 	dispatch::DispatchResult,
// // 	ensure,
// // 	traits::{
// // 		tokens::nonfungibles::{Create, Inspect, Mutate},
// // 		Get,
// // 	},
// // };
// // use scale_info::TypeInfo;
// // use sp_runtime::{DispatchError, TokenError, Permill};
// // use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

// // pub type AttributeKey = BiBoundedVec<u8, 1, 64>;
// // pub type AttributeValue = BiBoundedVec<u8, 1, 256>; 

// // pub trait FinancialNftProvider<AccountId>: Create<AccountId> + Mutate<AccountId> {
// // 	/// Mint an NFT instance with initial (key, value) attribute in the given account.
// // 	///
// // 	/// Arguments
// // 	///
// // 	/// * `class` the NFT class id.
// // 	/// * `who` the owner of the minted NFT.
// // 	/// * `key` the key of the initial attribute.
// // 	/// * `reference` the value of the initial attribute.
// // 	///
// // 	/// Note: we store the NFT scale encoded struct under a single attribute key.
// // 	///
// // 	/// Returns the unique instance id.
// // 	fn mint<K: Encode, V: Encode>(
// // 		who: &AccountId,

// // 		// unique identify NFT metadata storage
// // 		class: &Self::ClassId,
// // 		version: &K,
// // 		reference: &V,

// // 	) -> Result<Self::InstanceId, DispatchError>;

// // 	fn split(
// // 		instance: &Self::InstanceId,
// // 		parts: BiBoundedVec<Permill, 1, 16>,
// // 	) -> Result<BiBoundedVec<Self::InstanceId, 1, 16>, DispatchError>;

// // 	/// Retrieve the _possible_ owner of the NFT identified by `instance_id`.
// // 	///
// // 	/// Arguments
// // 	///
// // 	/// * `instance_id` the ID that uniquely identify the NFT.
// // 	fn get_owner<NFT>(
// // 		instance_id: &Self::InstanceId,
// // 	) -> Result<AccountId, DispatchError>
// // 	where
// // 		NFT: Get<Self::ClassId>,
// // 	{
// // 		Self::NFTProvider::owner(&NFT::get(), instance_id).ok_or(DispatchError::CannotLookup)
// // 	}

// // }

// // 	/// Ensure that the owner of the identifier NFT is `account_id`.
// // 	///
// // 	/// Arguments
// // 	///
// // 	/// * `owner` the account id that should own the NFT.
// // 	/// * `instance_id` the NFT instance id.
// // 	///
// // 	/// Returns `Ok(())` if `owner` is the owner of the NFT identified by `instance_id`.
// // 	fn ensure_protocol_nft_owner<NFT>(
// // 		owner: &AccountId,
// // 		instance_id: &Self::InstanceId,
// // 	) -> Result<(), DispatchError>
// // 	where
// // 		NFT: Get<Self::ClassId>,
// // 	{
// // 		let nft_owner = Self::get_protocol_nft_owner::<NFT>(instance_id)?;
// // 		ensure!(nft_owner == *owner, DispatchError::BadOrigin);
// // 		Ok(())
// // 	}




// // /// Default ClassId type used for NFTs.
// // #[derive(
// // 	Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, MaxEncodedLen, TypeInfo,
// // )]
// // #[repr(transparent)]
// // pub struct NftClass(u8);

// // #[cfg(feature = "test-utils")]
// // impl NftClass {
// // 	/// Create a new [`NftClass`].
// // 	///
// // 	/// Will not necessarilly be a well-known class; only for use in testing.
// // 	pub fn new(inner: u8) -> Self {
// // 		NftClass(inner)
// // 	}
// // }

// // impl NftClass {
// // 	pub const STAKING: NftClass = NftClass(1);
// // }

// // /// Default Version type used for NFTs.
// // #[derive(
// // 	Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, MaxEncodedLen, TypeInfo,
// // )]
// // #[repr(transparent)]
// // pub struct NftVersion(u8);

// // impl NftVersion {
// // 	pub const VERSION_1: NftVersion = NftVersion(1);
// // }
