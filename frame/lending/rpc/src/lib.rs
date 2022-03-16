use composable_support::rpc_helpers::{SafeRpcWrapper, SafeRpcWrapperType};
use frame_support::{pallet_prelude::MaybeSerializeDeserialize, Parameter};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result as RpcResult};
use jsonrpc_derive::rpc;
use lending_runtime_api::LendingRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::{marker::PhantomData, sync::Arc};

#[rpc]
pub trait LendingApi<BlockHash, AccountId, Balance, MarketId>
where
	Balance: SafeRpcWrapperType,
{
	#[rpc(name = "lending_getBorrowLimit")]
	fn get_borrow_limit(
		&self,
		market_id: MarketId,
		account: AccountId,
		at: Option<BlockHash>,
	) -> RpcResult<SafeRpcWrapper<Balance>>;
}

pub struct Lending<C, Block> {
	client: Arc<C>,
	_marker: PhantomData<Block>,
}

impl<C, M> Lending<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId, Balance, MarketId>
	LendingApi<<Block as BlockT>::Hash, AccountId, Balance, MarketId>
	for Lending<C, (Block, AccountId, Balance, MarketId)>
where
	Block: BlockT,
	AccountId: Send + Sync + Parameter + MaybeSerializeDeserialize + Ord + 'static,
	MarketId: Send + Sync + Parameter + MaybeSerializeDeserialize + Ord + 'static,
	Balance: Send + Sync + 'static + SafeRpcWrapperType,
	C: Send + Sync + ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	C::Api: LendingRuntimeApi<Block, AccountId, Balance, MarketId>,
{
	fn get_borrow_limit(
		&self,
		market_id: MarketId,
		account: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<SafeRpcWrapper<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| {
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash
		}));

		let runtime_api_result = api.get_borrow_limit(&at, market_id, account);
		// TODO(benluelo): Review what error message & code to use
		runtime_api_result.map_err(|e| {
			RpcError {
				code: ErrorCode::ServerError(9876), // No real reason for this value
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			}
		})
	}
}
