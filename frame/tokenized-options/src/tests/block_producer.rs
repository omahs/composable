use frame_support::{
	traits::{Get, OnFinalize, OnIdle, OnInitialize},
	weights::Weight,
};
use frame_system::{Config as SystemConfig, Pallet as SystemPallet};
use pallet_timestamp::{Config as TimestampConfig, Pallet as TimestampPallet};
use sp_runtime::traits::{CheckedAdd, One};

/// Block producer configuration trait.
pub trait BlocksConfig {
	/// Runtime type.
	type Runtime: pallet_timestamp::Config;

	/// Pallet type for which hooks are called.
	type Hooked: OnInitialize<BlockNumberOf<Self>>
		+ OnIdle<BlockNumberOf<Self>>
		+ OnFinalize<BlockNumberOf<Self>>;
}

pub type BlockNumberOf<C> = <<C as BlocksConfig>::Runtime as SystemConfig>::BlockNumber;
pub type MomentOf<C> = <<C as BlocksConfig>::Runtime as TimestampConfig>::Moment;

pub struct BlockProducer<C: BlocksConfig> {
	remaining_moments: std::vec::IntoIter<MomentOf<C>>,
	init: bool,
	finalized: bool,
	weight: Weight,
}

impl<C: BlocksConfig> std::fmt::Debug for BlockProducer<C> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		f.debug_struct("BlockProducer")
			.field("remaining_moments", &self.remaining_moments)
			.field("init", &self.init)
			.field("finalized", &self.finalized)
			.field("weight", &self.weight)
			.finish()
	}
}

impl<C: BlocksConfig> Drop for BlockProducer<C> {
	fn drop(&mut self) {
		self.finalize_block();
	}
}

impl<C: BlocksConfig> BlockProducer<C> {
	pub fn new(moments: Vec<MomentOf<C>>) -> Self {
		BlockProducer {
			remaining_moments: moments.into_iter(),
			init: true,
			finalized: true,
			weight: 0,
		}
	}
	pub fn block_number(&self) -> BlockNumberOf<C> {
		SystemPallet::<C::Runtime>::block_number()
	}
	pub fn remaining_moments(&self) -> &[MomentOf<C>] {
		self.remaining_moments.as_slice()
	}
	pub fn is_init_block(&self) -> bool {
		self.init
	}
	pub fn is_last_block(&self) -> bool {
		self.remaining_moments.len() == 0
	}
	pub fn weight(&self) -> Weight {
		self.weight
	}
	pub fn inc_weight(&mut self, weight: Weight) {
		self.weight = self.weight.saturating_add(weight);
	}
	pub fn next_block(&mut self) -> bool {
		self.finalize_block();
		match self.remaining_moments.next() {
			Some(moment) => {
				let block_number = self.block_number();
				let block_number = if self.init {
					block_number
				} else {
					block_number
						.checked_add(&BlockNumberOf::<C>::one())
						.expect("Hit the limit for block number!")
				};
				if !self.init {
					SystemPallet::<C::Runtime>::reset_events();
					SystemPallet::<C::Runtime>::set_block_number(block_number);
				}
				// TODO: move the next line after on_initialize and call on_post_inherent
				// when https://github.com/paritytech/substrate/pull/10128 is merged.
				TimestampPallet::<C::Runtime>::set_timestamp(moment);
				self.weight = C::Hooked::on_initialize(block_number);
				self.finalized = false;
				true
			},
			None => false,
		}
	}
	pub fn finalize_block(&mut self) {
		if self.finalized {
			return;
		}
		let max_weight = <C::Runtime as SystemConfig>::BlockWeights::get().max_block;
		let remaining_weight = if !self.is_last_block() {
			max_weight.saturating_sub(self.weight)
		} else {
			Weight::max_value()
		};
		let block_number = self.block_number();
		let idle_weight = C::Hooked::on_idle(block_number, remaining_weight);
		self.inc_weight(idle_weight);
		C::Hooked::on_finalize(block_number);
		self.init = false;
		self.finalized = true;
	}
}
