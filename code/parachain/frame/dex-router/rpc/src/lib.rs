use codec::Codec;
use composable_support::rpc_helpers::SafeRpcWrapper;
use composable_traits::dex::{PriceAggregate, RemoveLiquiditySimulationResult};
use core::{fmt::Display, str::FromStr};
use dex_router_runtime_api::DexRouterRuntimeApi;
use jsonrpsee::{
	core::{Error as RpcError, RpcResult},
	proc_macros::rpc,
	types::{error::CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::{cmp::Ord, collections::btree_map::BTreeMap, sync::Arc};

#[rpc(client, server)]
pub triat DexRouterApi {
	#[method(name = "dex_router_fee_for_indirect_pool")]
	fn fee_for_indirect_pool();
}

pub struct DexRouter<C, Block> {
	client: Arc<C>
	_marker: sp_std::marker::PhantomData<Block>,
}

impl<C, M> DexRouter<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block> DexRouterApiServer for DexRouter<C, (Block)>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: DexRouterRuntimeApi<Block>,
{
	fn fee_for_indirect_pool() {
		
	}
}
