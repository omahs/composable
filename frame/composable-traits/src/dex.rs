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
	/// The asset ID type
	type AssetId: AssetIdLike;
	/// The balance type of an account
	type Balance : BalanceLike;
	/// The user account identifier type for the runtime	
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


/// given `base`, how much `quote` needed for unit
/// see [currency pair](https://www.investopedia.com/terms/c/currencypair.asp)
pub struct CurrencyPair<AssetId> {
	pub base: AssetId,
	/// counter currency
	pub quote: AssetId,
}

/// take `quote` currency and give `base` currency
pub struct Sell<AssetId, Price> {
	pub pair: CurrencyPair<AssetId>,
	/// minimal amount of `quote` for given unit of `base` 
	pub amount: Price,
}

/// take `base` currency and give `quote` currency back
pub struct Buy<AssetId, Balance> {
	pub pair: CurrencyPair<AssetId>,
	/// maximal price of `base` in `quote` 
	pub amount: Price,
}

/// This order book is not fully DEX as it has no matching engine.
/// How to sell in market price using this orderbook? 
/// Request existing orders summary and send with `ask`/`bid` with proper amount. 
/// Or create new trait which is market aware, market sell api.
/// How to I see success for my operations?
/// Observer events or on chain history or your account state for give currency.
pub trait LimitOrderbook : DeFiTrait {
	type OrderId;
	/// if there is AMM,  and [Self::AmmConfiguration] allows for that, than can use DEX to sell some amount if it is good enough
	type AmmDex;
	/// amm configuration parameter
	type AmmConfiguration;
	/// sell for price given or higher 
	/// `account_from` - account requesting sell 
	fn ask(
		account_from: &Self::AccountId,
		order: Sell<Self::AssetId, Self::Balance>,		
		in_amount: Self::Balance,
		amm: AmmConfiguration,
	) -> Result<Self::OrderId, DispatchError>;

	///  buy for price given or lower
	fn bid(
		account_from: &Self::AccountId,
		order: Buy<Self::AssetId, Self::Balance>,		
		in_amount: Self::Balance,
		amm: AmmConfiguration,
	) -> Result<Self::OrderId, DispatchError>;

	/// updates same existing order with new price
	/// to avoid overpay, use `take` with `up_to` price
	fn patch(
		order_id: Self::OrderId,
		price: Self::Balance,
	) -> Result<(), DispatchError>;
	
	/// take order. get not found error if order never existed or was removed.
	/// `price` - for `sell` order it is maximal value are you to pay for `base`, for `buy` order it is minimal value you are eager to accept for `base`
	/// `amount` - amount of `base` you are ready to exchange for this order
	fn take(
		account: &Self::AccountId,
		order: Self::OrderId,
		amount : Self::Balance,
		price: Self::Balance,
	) -> Result<(), DispatchError>;
}

/// Implement AMM curve from [StableSwap - efficient mechanism for Stablecoin liquidity by Micheal Egorov](https://curve.fi/files/stableswap-paper.pdf) 
/// Also blog at [Understanding stableswap curve](https://miguelmota.com/blog/understanding-stableswap-curve/) as explanation.
pub trait CurveAmm : DeFiTrait {
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
