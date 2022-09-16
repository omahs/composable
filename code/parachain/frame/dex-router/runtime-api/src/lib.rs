#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use composable_support::rpc_helpers::SafeRpcWrapper;
use composable_traits::dex::{PriceAggregate, RemoveLiquiditySimulationResult};
use sp_std::collections::btree_map::BTreeMap;

// Dex Rounter Runtime API declaration. Implemented for each runtime at
// `runtime/<runtime-name>/src/lib.rs`.
sp_api::decl_runtime_apis! {
	pub trait DexRouterRuntimeApi {
		fn fee_for_indirect_pool();
	}
}
