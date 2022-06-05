pub mod protocol;


use codec::{Decode, Encode, FullCodec, MaxEncodedLen};
use composable_support::collections::vec::bounded::BiBoundedVec;
use core::fmt::Debug;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{
		tokens::nonfungibles::{Create, Inspect, Mutate},
		Get,
	},
};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, TokenError, Permill};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

pub type AttributeKey = BiBoundedVec<u8, 1, 64>;
pub type AttributeValue = BiBoundedVec<u8, 1, 256>; 

pub trait FinancialNftProvider<AccountId>: Create<AccountId> + Mutate<AccountId> + Inspect<AccountId> {
    fn protocol_mint_into<NFTProvider, NFT>(
		_class: &Self::ClassId,
		_instance: &Self::InstanceId,
        nft: NFT,
		_who: &AccountId,
	) -> DispatchResult;


    fn protocol_split_into<NFTProvider, NFT>(
		instance: &Self::InstanceId,
		parts: BiBoundedVec<Permill, 1, 16>,
	) -> Result<BiBoundedVec<Self::InstanceId, 1, 16>, DispatchError>;
}


/// Default ClassId type used for NFTs.
#[derive(
	Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, MaxEncodedLen, TypeInfo,
)]
#[repr(transparent)]
pub struct NftClass(u8);

#[cfg(feature = "test-utils")]
impl NftClass {
	/// Create a new [`NftClass`].
	///
	/// Will not necessarilly be a well-known class; only for use in testing.
	pub fn new(inner: u8) -> Self {
		NftClass(inner)
	}
}

impl NftClass {
	pub const STAKING: NftClass = NftClass(1);
}

/// Default Version type used for NFTs.
#[derive(
	Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, MaxEncodedLen, TypeInfo,
)]
#[repr(transparent)]
pub struct NftVersion(u8);

impl NftVersion {
	pub const VERSION_1: NftVersion = NftVersion(1);
}
