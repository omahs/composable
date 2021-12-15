use crate::{
	dex::{LimitOrderbook},
	loans::{DurationSeconds, Timestamp}, currency::{AssetIdLike, BalanceLike}, defi::{DeFiTrait, Sell, SellTrait},
};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::Permill;

#[derive(Decode, Encode, Clone, TypeInfo)]
pub enum AuctionStepFunction {
	/// default - direct pass through to dex without steps, just to satisfy defaults and reasonably
	/// for testing
	LinearDecrease(LinearDecrease),
	StairstepExponentialDecrease(StairstepExponentialDecrease),
}

impl Default for AuctionStepFunction {
	fn default() -> Self {
		Self::LinearDecrease(Default::default())
	}
}

#[derive(Decode, Encode, Clone, PartialEq, TypeInfo)]
pub enum AuctionState<DexOrderId> {
	AuctionStarted,
	AuctionOnDex(DexOrderId),
	AuctionEndedSuccessfully,
	/// like DEX does not support asset now or halted
	AuctionFatalFailed,
	/// so if for some reason system loop is not properly set, than will get timeout
	AuctionTimeFailed,
}

impl<DexOrderId> Default for AuctionState<DexOrderId> {
	fn default() -> Self {
		Self::AuctionStarted
	}
}

#[derive(Default, Decode, Encode, Clone, TypeInfo)]
pub struct LinearDecrease {
	/// Seconds after auction start when the price reaches zero
	pub total: DurationSeconds,
}


#[derive(Default, Decode, Encode, Clone, TypeInfo)]
pub struct StairstepExponentialDecrease {
	// Length of time between price drops
	pub step: DurationSeconds,
	// Per-step multiplicative factor, usually more than 50%, mostly closer to 100%, but not 100%.
	// Drop per unit of `step`.
	pub cut: Permill,
}

/// see example of it in clip.sol of makerdao
pub trait DutchAuction : SellTrait<AuctionStepFunction> {
	type Order;
	fn get_order(order: &Self::OrderId) -> Option<Self::Order>;
}
