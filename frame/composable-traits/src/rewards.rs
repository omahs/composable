use crate::{
	financial_nft::{NftClass, NftVersion},
	time::{DurationSeconds, Timestamp},
};
use codec::{Decode, Encode};
use composable_support::math::safe::SafeSub;
use core::fmt::Debug;
use frame_support::{
	dispatch::DispatchResult, storage::bounded_btree_map::BoundedBTreeMap, traits::Get,
};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Saturating, Zero},
	DispatchError, Perbill,
};


pub type DurationMultiplierRewardsConfig<Limit:Get<u32>> = BoundedBTreeMap<DurationSeconds, Perbill, Limit>;

/// staking typed fNFT, usually can be mapped to raw fNFT storage type
#[derive(Debug, PartialEq, Eq, Copy, Clone, Encode, Decode, TypeInfo)]
pub struct Rewards<RewardsUpdates> {
	/// List of reward asset/pending rewards.
	pub rewards: RewardsUpdates,
	/// The reward multiplier. Captured from `RewardsConfig`  on creation.
	pub reward_multiplier: Perbill,
}
