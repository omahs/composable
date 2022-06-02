use crate::{self as composable_traits};

use composable_traits::{
	financial_nft::{NftClass, NftVersion},
	time::{DurationSeconds, Timestamp}, lock::{LockConfig, Lock,},
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


#[derive(Debug, PartialEq, Eq, Copy, Clone, Encode, Decode, TypeInfo)]
pub struct StakeConfig<DurationPresets> {
	/// your stake make lock to get rumpup
	pub lock : Option<LockConfig<DurationPresets>>,
}

/// staking typed fNFT, usually can be mapped to raw fNFT storage type
#[derive(Debug, PartialEq, Eq, Copy, Clone, Encode, Decode, TypeInfo)]
pub struct Stake< Balance,  Rewards> {
	/// The original stake this NFT was minted for or updated NFT with increased stake amount.
	pub real_stake: Balance,
	/// List of reward asset/pending rewards.
	pub rewards: Rewards,
	pub lock: Option<Lock>,	
	/// The reward multiplier.
	pub reward_multiplier: Perbill,
}

/// implemented by instances which know their share of something bigger
pub trait Shares {
	type Balance;
	fn shares(&self) -> Self::Balance;
}

// impl<AccountId, AssetId, Balance: AtLeast32BitUnsigned + Copy, Epoch: Ord, Rewards>
// 	StakingNFT<AccountId, AssetId, Balance, Epoch, Rewards>
// {
// 	pub fn state(&self, epoch: &Epoch, epoch_start: Timestamp) -> PositionState {
// 		if self.lock_date.saturating_add(self.lock_duration) < epoch_start {
// 			PositionState::Expired
// 		} else if self.reward_epoch_start > *epoch {
// 			PositionState::Pending
// 		} else {
// 			PositionState::LockedRewarding
// 		}
// 	}
// }

// impl<
// 		AccountId,
// 		AssetId: PartialEq + Ord,
// 		Balance: AtLeast32BitUnsigned + Copy + Zero + Saturating,
// 		Epoch: Ord,
// 		S: frame_support::traits::Get<u32>,
// 	> Shares for StakingNFT<AccountId, AssetId, Balance, Epoch, BoundedBTreeMap<AssetId, Balance, S>>
// {
// 	type Balance = Balance;
// 	fn shares(&self) -> Balance {
// 		let compound = *self.pending_rewards.get(&self.asset).unwrap_or(&Balance::zero());
// 		self.reward_multiplier.mul_floor(self.stake).saturating_add(compound)
// 	}
// }

// impl<AccountId, AssetId, Balance, Epoch, Rewards> Get<NftClass>
// 	for Stake<AccountId, AssetId, Balance, Epoch, Rewards>
// {
// 	fn get() -> NftClass {
// 		NftClass::STAKING
// 	}
// }

// impl<AccountId, AssetId, Balance, Epoch, Rewards> Get<NftVersion>
// 	for StakingNFT<AccountId, AssetId, Balance, Epoch, Rewards>
// {
// 	fn get() -> NftVersion {
// 		NftVersion::VERSION_1
// 	}
// }

// /// Interface for protocol staking.
// pub trait Staking {
// 	type AccountId;
// 	type AssetId;
// 	type Balance;
// 	type InstanceId;

// 	/// Stake an amount of protocol asset. A new NFT representing the user position will be
// 	/// minted.
// 	///
// 	/// Arguments
// 	///
// 	/// * `asset` the protocol asset to stake.
// 	/// * `from` the account to transfer the stake from.
// 	/// * `amount` the amount to stake.
// 	/// * `duration` the staking duration (must be one of the predefined presets). Unstaking before
// 	///   the end trigger the unstake penalty.
// 	/// * `keep_alive` whether to keep the `from` account alive or not while transferring the stake.
// 	fn stake(
// 		asset: &Self::AssetId,
// 		from: &Self::AccountId,
// 		amount: Self::Balance,
// 		duration: DurationSeconds,
// 		keep_alive: bool,
// 	) -> Result<Self::InstanceId, DispatchError>;

// 	/// Unstake an actual staked position, represented by a NFT.
// 	///
// 	/// Arguments
// 	///
// 	/// * `instance_id` the ID uniquely identifiying the NFT from which we will compute the
// 	///   available rewards.
// 	/// * `to` the account to transfer the final claimed rewards to.
// 	fn unstake(instance_id: &Self::InstanceId, to: &Self::AccountId) -> DispatchResult;

// 	/// Claim the current rewards.
// 	///
// 	/// Arguments
// 	///
// 	/// * `who` the actual account triggering this claim.
// 	/// * `instance_id` the ID uniquely identifiying the NFT from which we will compute the
// 	///   available rewards.
// 	/// * `to` the account to transfer the rewards to.
// 	/// Return amount if reward asset which was staked asset claimed.
// 	fn claim(
// 		instance_id: &Self::InstanceId,
// 		to: &Self::AccountId,
// 	) -> Result<(Self::AssetId, Self::Balance), DispatchError>;
// }

// pub trait StakingReward {
// 	type AccountId;
// 	type AssetId;
// 	type Balance;

// 	/// Transfer a reward to the staking rewards protocol.
// 	///
// 	/// Arguments
// 	///
// 	/// * `asset` the protocol asset to reward.
// 	/// * `reward_asset` the reward asset to transfer.
// 	/// * `from` the account to transfer the reward from.
// 	/// * `amount` the amount of reward to transfer.
// 	/// * `keep_alive` whether to keep alive or not the `from` account while transferring the
// 	///   reward.
// 	fn transfer_reward(
// 		asset: &Self::AssetId,
// 		reward_asset: &Self::AssetId,
// 		from: &Self::AccountId,
// 		amount: Self::Balance,
// 		keep_alive: bool,
// 	) -> DispatchResult;
// }
