
//! Autogenerated weights for `collective`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-18, STEPS: `50`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `11f0030002e5`, CPU: `Intel(R) Xeon(R) CPU @ 3.10GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("composable-dev"), DB CACHE: 1024

// Executed Command:
// /nix/store/a8zw7i97gjsaanq9c839pbaklnsjsqcf-composable/bin/composable
// benchmark
// pallet
// --chain=composable-dev
// --execution=wasm
// --wasm-execution=compiled
// --wasm-instantiation-strategy=legacy-instance-reuse
// --pallet=*
// --extrinsic=*
// --steps=50
// --repeat=10
// --output=code/parachain/runtime/composable/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> collective::WeightInfo for WeightInfo<T> {
	// Storage: Council Members (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Voting (r:100 w:100)
	// Storage: Council Prime (r:0 w:1)
	/// The range of component `m` is `[1, 100]`.
	/// The range of component `n` is `[1, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(69_130_000 as u64)
			// Standard Error: 130_646
			.saturating_add(Weight::from_ref_time(8_236_848 as u64).saturating_mul(m as u64))
			// Standard Error: 130_646
			.saturating_add(Weight::from_ref_time(11_985_174 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().reads((1 as u64).saturating_mul(p as u64)))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
			.saturating_add(T::DbWeight::get().writes((1 as u64).saturating_mul(p as u64)))
	}
	// Storage: Council Members (r:1 w:0)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		Weight::from_ref_time(35_281_000 as u64)
			// Standard Error: 114
			.saturating_add(Weight::from_ref_time(1_373 as u64).saturating_mul(b as u64))
			// Standard Error: 1_172
			.saturating_add(Weight::from_ref_time(43_731 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:0)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		Weight::from_ref_time(38_622_000 as u64)
			// Standard Error: 94
			.saturating_add(Weight::from_ref_time(1_012 as u64).saturating_mul(b as u64))
			// Standard Error: 966
			.saturating_add(Weight::from_ref_time(83_048 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalCount (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(52_660_000 as u64)
			// Standard Error: 313
			.saturating_add(Weight::from_ref_time(5_348 as u64).saturating_mul(b as u64))
			// Standard Error: 3_236
			.saturating_add(Weight::from_ref_time(43_759 as u64).saturating_mul(m as u64))
			// Standard Error: 3_217
			.saturating_add(Weight::from_ref_time(464_899 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Voting (r:1 w:1)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		Weight::from_ref_time(60_617_000 as u64)
			// Standard Error: 2_879
			.saturating_add(Weight::from_ref_time(246_945 as u64).saturating_mul(m as u64))
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(60_055_000 as u64)
			// Standard Error: 2_248
			.saturating_add(Weight::from_ref_time(59_486 as u64).saturating_mul(m as u64))
			// Standard Error: 2_258
			.saturating_add(Weight::from_ref_time(429_495 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(78_903_000 as u64)
			// Standard Error: 383
			.saturating_add(Weight::from_ref_time(1_786 as u64).saturating_mul(b as u64))
			// Standard Error: 3_976
			.saturating_add(Weight::from_ref_time(84_835 as u64).saturating_mul(m as u64))
			// Standard Error: 3_933
			.saturating_add(Weight::from_ref_time(455_990 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(62_609_000 as u64)
			// Standard Error: 1_906
			.saturating_add(Weight::from_ref_time(67_815 as u64).saturating_mul(m as u64))
			// Standard Error: 1_915
			.saturating_add(Weight::from_ref_time(427_505 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Council Voting (r:1 w:1)
	// Storage: Council Members (r:1 w:0)
	// Storage: Council Prime (r:1 w:0)
	// Storage: Council ProposalOf (r:1 w:1)
	// Storage: Council Proposals (r:1 w:1)
	/// The range of component `b` is `[1, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		Weight::from_ref_time(85_157_000 as u64)
			// Standard Error: 307
			.saturating_add(Weight::from_ref_time(3_129 as u64).saturating_mul(b as u64))
			// Standard Error: 3_193
			.saturating_add(Weight::from_ref_time(64_717 as u64).saturating_mul(m as u64))
			// Standard Error: 3_158
			.saturating_add(Weight::from_ref_time(405_214 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	// Storage: Council Proposals (r:1 w:1)
	// Storage: Council Voting (r:0 w:1)
	// Storage: Council ProposalOf (r:0 w:1)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		Weight::from_ref_time(34_141_000 as u64)
			// Standard Error: 3_378
			.saturating_add(Weight::from_ref_time(514_083 as u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
}
