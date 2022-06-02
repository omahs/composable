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


// /// The outcome of a penalty applied/notapplied to an amount.
// #[derive(Debug, PartialEq, Eq, Copy, Clone, Encode, Decode, TypeInfo)]
// pub enum PenaltyOutcome<AccountId, Balance> {
// 	/// The penalty has been actually applied.
// 	Applied {
// 		/// The amount remaining after having subtracted the penalty.
// 		amount_remaining: Balance,
// 		/// The penalty amount, a fraction of the amount we penalized (i.e. amount_remaining +
// 		/// amount_penalty = amount_to_penalize).
// 		amount_penalty: Balance,
// 		/// The beneficiary of the applied penalty.
// 		penalty_beneficiary: AccountId,
// 	},
// 	/// The penalty has not beend applied, i.e. identity => f x = x.
// 	NotApplied { amount: Balance },
// }

// impl<AccountId, Balance: Zero + Copy> PenaltyOutcome<AccountId, Balance> {
// 	pub fn penalty_amount(&self) -> Option<Balance> {
// 		match self {
// 			PenaltyOutcome::Applied { amount_penalty, .. } => Some(*amount_penalty),
// 			PenaltyOutcome::NotApplied { .. } => None,
// 		}
// 	}

// 	// NOTE(hussein-aitlahcen): sadly, Zero is asking for Add<Output = Self> for no particular
// 	// reason?
// 	pub fn is_zero(&self) -> bool {
// 		match self {
// 			PenaltyOutcome::Applied { amount_remaining, amount_penalty, .. } =>
// 				amount_remaining.is_zero() && amount_penalty.is_zero(),
// 			PenaltyOutcome::NotApplied { amount } => amount.is_zero(),
// 		}
// 	}
// }


// impl<AccountId: Clone> Penalty<AccountId> {
// 	pub fn penalize<Balance>(
// 		&self,
// 		amount: Balance,
// 	) -> Result<PenaltyOutcome<AccountId, Balance>, DispatchError>
// 	where
// 		Balance: AtLeast32BitUnsigned + Copy,
// 	{
// 		if self.value.is_zero() {
// 			Ok(PenaltyOutcome::NotApplied { amount })
// 		} else {
// 			let amount_penalty = self.value.mul_floor(amount);
// 			let amount_remaining = amount.safe_sub(&amount_penalty)?;
// 			Ok(PenaltyOutcome::Applied {
// 				amount_penalty,
// 				amount_remaining,
// 				penalty_beneficiary: self.beneficiary.clone(),
// 			})
// 		}
// 	}
// }

/// defines staking duration, rewards and early unstake penalty
#[derive(Debug, PartialEq, Eq, Copy, Clone, Encode, Decode, TypeInfo)]
pub struct LockConfig<AccountId, DurationPresets, RewardAssets> {
	/// The possible locking duration.
	pub duration_presets: DurationPresets,
	/// The penalty applied if a staker unstake before the end date.
	/// In case of zero penalty, you cannot unlock before it duration ends.
	pub unlock_penalty:  Perbill,
}