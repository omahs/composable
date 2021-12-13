#![allow(dead_code)]
#![allow(clippy::many_single_char_names)]
use frame_support::{
	codec::{Decode, Encode},
	sp_runtime::Perbill,
};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, Permill};

use sp_std::vec::Vec;

/// type parameters for traits in pure defi area
pub trait DeFiTrait {
	type AssetId: AssetIdLike;
	type Balance : BalanceLike;
	type AccountId;
}

/// Immediate AMM exchange. Either resolves trade immediately or returns error (mostly because of lack of liquidity).
pub trait AmmExchange : DeFiTrait {	
	type Error;

	/// Obtains the current price for a given asset, possibly routing through multiple markets.
	fn price(asset_id: Self::AssetId) -> Option<Self::Balance>;

	/// Exchange `amount` of `from` asset for `to` asset. The maximum price paid for the `to` asset
	/// is `SimpleExchange::price * (1 + slippage)`
	fn exchange(
		from: Self::AssetId,
		from_account: Self::AccountId,
		to: Self::AssetId,
		to_account: Self::AccountId,
		to_amount: Self::Balance,
		slippage: Perbill,
	) -> Result<Self::Balance, DispatchError>;
}

pub struct TakeResult<BALANCE> {
	pub amount: BALANCE,
	pub total_price: BALANCE,
}

pub struct SellOrder<OrderId, AccountId> {
	pub id: OrderId,
	/// account holding sell order amount.
	/// if it becomes empty or non existing, and there was no direct call from seller to cancel
	/// order, it means amount was sold
	pub account: AccountId,
}

#[derive(Encode, Decode)]
pub enum Price<GroupId, Balance> {
	Preferred(GroupId, Balance),
	Both { preferred_id: GroupId, preferred_price: Balance, any_price: Balance },
	Any(Balance),
}

impl<GroupId, Balance> Price<GroupId, Balance> {
	pub fn new_any(price: Balance) -> Self {
		Self::Any(price)
	}
}

// orderdboox dex = amm + order book + matcher
// amm - to sell if price is better or to sell after failed ob sell with some slippages
// orderbook - can be fully  off chain (i did not found at all in hydra storage in their dex pallet - so store order only for one block and delete on finalization),  or on chain
// matchmaker  - can operate only if there is off chain component (so it matches only there orders which likely to success onto onchain)
// all 3 ob work like that (hydra, polkadex - closed source, only as per docs, and examples from solidity)
// matcher can be of different logic - who is served first? biggest ask/bid, fifo, etc...
// sell - i have exactly X and can receive approximately Y. and buy - i want exactly Y, can spend approximately X. so these are very symmetrical up to slippage.
// thats is by order book can be 2 collection of sell and buy by asset id, or it can be one collection of intentions (from, too) => amount, type.
//  9. i tried to find and read code of more on chain order books, like solana serum, but their codebase and patters are way complicated (but seems cool)
//  10. hydradx code is opinionated about matched order priority, not sure if that is good order.
// so for liqudations ordebook is very simple, just sells and buys, and any caller from on chain can take any of these if observers  good position. no matcher on chain.
// documing all this along the way.

// so we have simple on chain order book with external matcher (anybody can observer and take)


/// This order book is not fully DEX as it has not matching engine.
pub trait SimpleOrderBook : DeFiTrait {

}

/// see for examples:
/// - https://github.com/galacticcouncil/Basilisk-node/blob/master/pallets/exchange/src/lib.rs
/// - https://github.com/Polkadex-Substrate/polkadex-aura-node/blob/master/pallets/polkadex/src/lib.rs
/// expected that failed exchanges are notified by events.
pub trait OrderbookDex: DeFiTrait {
	type OrderId;
	type GroupId;

	/// Sell. exchanges specified amount of asset to other at specific price
	/// `source_total_price` normalized
	/// `amm_slippage` set to zero to avoid AMM sell
	/// for remote auction we should  have sent some random to make sure we have idempotent request
	fn post(
		account_from: &Self::AccountId,
		asset: Self::AssetId,
		want: Self::AssetId,
		source_amount: Self::Balance,
		source_total_price: Price<Self::GroupId, Self::Balance>,
		amm_slippage: Permill,
	) -> Result<SellOrder<Self::OrderId, Self::AccountId>, DispatchError>;

	/// updates same existing order with new price
	/// to avoid overpay, use `take` with `up_to` price
	fn patch(
		order_id: Self::OrderId,
		price: Price<Self::GroupId, Self::Balance>,
	) -> Result<(), DispatchError>;

	/// sell. exchanges specified amount of asset to other at market price.
	/// `amm_slippage` - allow maxima slippage of order
	fn market_sell(
		account: &Self::AccountId,
		asset: Self::AssetId,
		want: Self::AssetId,
		amount: Self::Balance,
		amm_slippage: Permill,
	) -> Result<Self::OrderId, DispatchError>;

	/// ask to take order. get not found error if order never existed or was removed. got conflict
	/// error if order still on chain but was executed. please subscribe to events dispatched or
	/// check your balance or check blockchain history to validate your won the order.
	fn ask(
		account: &Self::AccountId,
		orders: impl Iterator<Item = Self::OrderId>,
		up_to: Self::Balance,
	) -> Result<(), DispatchError>;
}



/// Implement AMM curve from "StableSwap - efficient mechanism for Stablecoin liquidity by Micheal
/// Egorov" Also blog at https://miguelmota.com/blog/understanding-stableswap-curve/ has very good explanation.
pub trait CurveAmm {
	/// The asset ID type
	type AssetId;
	/// The balance type of an account
	type Balance;
	/// The user account identifier type for the runtime
	type AccountId;

	/// Current number of pools (also ID for the next created pool)
	fn pool_count() -> PoolId;

	/// Information about the pool with the specified `id`
	fn pool(id: PoolId) -> Option<PoolInfo<Self::AccountId, Self::AssetId, Self::Balance>>;

	/// Creates a pool, taking a creation fee from the caller
	fn create_pool(
		who: &Self::AccountId,
		assets: Vec<Self::AssetId>,
		amplification_coefficient: Self::Balance,
		fee: Permill,
		admin_fee: Permill,
	) -> Result<PoolId, DispatchError>;

	/// Deposit coins into the pool
	/// `amounts` - list of amounts of coins to deposit,
	/// `min_mint_amount` - minimum amout of LP tokens to mint from the deposit.
	fn add_liquidity(
		who: &Self::AccountId,
		pool_id: PoolId,
		amounts: Vec<Self::Balance>,
		min_mint_amount: Self::Balance,
	) -> Result<(), DispatchError>;

	/// Withdraw coins from the pool.
	/// Withdrawal amount are based on current deposit ratios.
	/// `amount` - quantity of LP tokens to burn in the withdrawal,
	/// `min_amounts` - minimum amounts of underlying coins to receive.
	fn remove_liquidity(
		who: &Self::AccountId,
		pool_id: PoolId,
		amount: Self::Balance,
		min_amounts: Vec<Self::Balance>,
	) -> Result<(), DispatchError>;

	/// Perform an exchange between two coins.
	/// `i` - index value of the coin to send,
	/// `j` - index value of the coin to receive,
	/// `dx` - amount of `i` being exchanged,
	/// `min_dy` - minimum amount of `j` to receive.
	fn exchange(
		who: &Self::AccountId,
		pool_id: PoolId,
		i: PoolTokenIndex,
		j: PoolTokenIndex,
		dx: Self::Balance,
		min_dy: Self::Balance,
	) -> Result<(), DispatchError>;

	/// Withdraw admin fees
	fn withdraw_admin_fees(
		who: &Self::AccountId,
		pool_id: PoolId,
		admin_fee_account: &Self::AccountId,
	) -> Result<(), DispatchError>;
}

/// Type that represents index type of token in the pool passed from the outside as an extrinsic
/// argument.
pub type PoolTokenIndex = u32;

/// Type that represents pool id
pub type PoolId = u32;

/// Pool type
#[derive(Encode, Decode, TypeInfo, Clone, Default, PartialEq, Eq, Debug)]
pub struct PoolInfo<AccountId, AssetId, Balance> {
	/// Owner of pool
	pub owner: AccountId,
	/// LP multiasset
	pub pool_asset: AssetId,
	/// List of multiasset supported by the pool
	pub assets: Vec<AssetId>,
	/// Initial amplification coefficient
	pub amplification_coefficient: Balance,
	/// Amount of the fee pool charges for the exchange
	pub fee: Permill,
	/// Amount of the admin fee pool charges for the exchange
	pub admin_fee: Permill,
	/// Current balances excluding admin_fee
	pub balances: Vec<Balance>,
	/// Current balances including admin_fee
	pub total_balances: Vec<Balance>,
}
