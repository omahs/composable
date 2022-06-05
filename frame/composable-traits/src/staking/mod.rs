
use crate::{self as composable_traits, };

use composable_traits::{
	staking::lock::{LockConfig, Lock,},
};
use codec::{Decode, Encode};

use core::fmt::Debug;
use frame_support::{
	dispatch::DispatchResult,
};
use scale_info::TypeInfo;
use sp_runtime::{
	DispatchError, Perbill,
};

pub mod lock;
pub mod math;
pub mod nft;
pub mod rewards;


#[derive(Debug, PartialEq, Eq, Copy, Clone, Encode, Decode, TypeInfo)]
pub struct StakeConfig<DurationPresets> {
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

pub trait ProtocolStaking {
	type AccountId;
	type AssetId;
	type Balance;
	type InstanceId;
	type PoolId;

	/// adds reward to common pool share
	fn accumulate_reward(
		pool: &Self::PoolId,
		reward_currency: Self::AssetId,
		reward_increment: Self::Balance,
	) -> DispatchResult;
}

/// Interface for protocol staking.
pub trait Staking {
	type AccountId;
	type PoolId;
	type Balance;
	type InstanceId;

	/// Stake an amount of protocol asset. A new NFT representing the user position will be
	/// minted.
	///
	/// Arguments
	///
	/// * `asset` the protocol asset to stake.
	/// * `from` the account to transfer the stake from.
	/// * `amount` the amount to stake.
	/// * `duration` the staking duration (must be one of the predefined presets). Unstaking before
	///   the end trigger the unstake penalty.
	/// * `keep_alive` whether to keep the `from` account alive or not while transferring the stake.
	/// * `config_index` config index to choose for stake
	fn create_share(
		who: &Self::AccountId,
		pool: &Self::PoolId,
		config_index: u8,
		add_amount: Self::Balance,
		keep_alive: bool,		
	) -> Result<Self::InstanceId, DispatchError>;

	fn add_share(who: &Self::AccountId, position: Self::InstanceId, add_amount: Self::Balance,  keep_alive: bool);

	/// Unstake an actual staked position, represented by a NFT.
	///
	/// Arguments
	///
	/// * `instance_id` the ID uniquely identifiying the NFT from which we will compute the
	///   available rewards.
	/// * `to` the account to transfer the final claimed rewards to.
	fn remove_share(
		who: &Self::AccountId, instance_id: &Self::InstanceId, remove_amount: Self::Balance) -> DispatchResult;

		// /// Splits fNFT position into several chunks with various amounts, but with same exposure.
		// /// fNFT splitted earns reward in current epoch proportional to split.
		// /// Can split only at  `State::WaitingForEpochEnd` state.
		// ///
		// /// `origin` - owner of fNFT
		// /// `amounts` - amount of in each fNFT, sum must equal to current stake.
		// ///
		// ///  raises event of NFT `SplitCreation`
		// // #[pallet::weight(10_000)]
		// // pub fn split(
		// // 	_origin: OriginFor<T>,
		// // 	_asset: InstanceIdOf<T>,
		// // 	_amounts: BiBoundedVec<T::Balance, 2, 16>,
		// // ) -> DispatchResult {
		// // 	Err(DispatchError::Other("no implemented. TODO: call split on fnft provider"))
		// // }

}


pub trait Locking {
	// #[pallet::weight(10_000)]
		// #[transactional]
		// pub fn extend_duration(
		// 	origin: OriginFor<T>,
		// 	instance_id: InstanceIdOf<T>,
		// 	duration: Option<DurationSeconds>,
		// ) -> DispatchResult {
}

pub trait StakingReward {
	type AccountId;
	type AssetId;
	type Balance;
	type InstanceId;


	/// Claim the current rewards.
	///
	/// Arguments
	///
	/// * `who` the actual account triggering this claim.
	/// * `instance_id` the ID uniquely identifiying the NFT from which we will compute the
	///   available rewards.
	/// * `to` the account to transfer the rewards to.
	/// Return amount if reward asset which was staked asset claimed.
	fn claim_rewards(
		who: &Self::AccountId,
		instance_id: &Self::InstanceId,
	) -> Result<(Self::AssetId, Self::Balance), DispatchError>;	

	/// Transfer a reward to the staking rewards protocol.
	///
	/// Arguments
	///
	/// * `asset` the protocol asset to reward.
	/// * `reward_asset` the reward asset to transfer.
	/// * `from` the account to transfer the reward from.
	/// * `amount` the amount of reward to transfer.
	/// * `keep_alive` whether to keep alive or not the `from` account while transferring the
	///   reward.
	fn claim_reward(
		who: &Self::AccountId,
		instance_id: &Self::InstanceId,
		amount: Self::Balance,
		keep_alive: bool,
	) -> DispatchResult;
}
