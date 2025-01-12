
//! Autogenerated weights for `treasury`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-12-16, STEPS: `50`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `c93baf6406af`, CPU: `Intel(R) Xeon(R) CPU @ 2.20GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("picasso-dev"), DB CACHE: 1024

// Executed Command:
// /nix/store/y1z2mfgy9msqas77hhxszf78hqg6mx5y-composable/bin/composable
// benchmark
// pallet
// --chain=picasso-dev
// --execution=wasm
// --wasm-execution=compiled
// --wasm-instantiation-strategy=legacy-instance-reuse
// --pallet=*
// --extrinsic=*
// --steps=50
// --repeat=10
// --output=code/parachain/runtime/picasso/src/weights

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `treasury`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> treasury::WeightInfo for WeightInfo<T> {
	fn spend() -> Weight {
		Weight::from_ref_time(563_000_u64)
	}
	// Storage: Treasury ProposalCount (r:1 w:1)
	// Storage: Treasury Proposals (r:0 w:1)
	fn propose_spend() -> Weight {
		Weight::from_ref_time(71_290_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	// Storage: Treasury Proposals (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	fn reject_proposal() -> Weight {
		Weight::from_ref_time(100_117_000_u64)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	// Storage: Treasury Proposals (r:1 w:0)
	// Storage: Treasury Approvals (r:1 w:1)
	/// The range of component `p` is `[0, 29]`.
	fn approve_proposal(p: u32, ) -> Weight {
		Weight::from_ref_time(30_247_000_u64)
			// Standard Error: 19_000
			.saturating_add(Weight::from_ref_time(460_000_u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: Treasury Approvals (r:1 w:1)
	fn remove_approval() -> Weight {
		Weight::from_ref_time(21_056_000_u64)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	// Storage: System Account (r:1 w:0)
	// Storage: Treasury Approvals (r:1 w:1)
	// Storage: Treasury Proposals (r:1 w:1)
	/// The range of component `p` is `[0, 30]`.
	fn on_initialize_proposals(p: u32, ) -> Weight {
		Weight::from_ref_time(80_249_000_u64)
			// Standard Error: 115_000
			.saturating_add(Weight::from_ref_time(77_193_000_u64).saturating_mul(p as u64))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(p as u64)))
			.saturating_add(T::DbWeight::get().writes(2_u64))
			.saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(p as u64)))
	}
}
