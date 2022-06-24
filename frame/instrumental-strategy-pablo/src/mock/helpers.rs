use composable_traits::{defi::CurrencyPair, dex::Amm};
use frame_support::{assert_ok, traits::fungibles::Mutate};
use pallet_pablo::PoolInitConfiguration;
use primitives::currency::CurrencyId;
use sp_runtime::Permill;

use crate::mock::{
	account_id::{AccountId, ALICE, BOB},
	runtime::{Balance, BlockNumber, Pablo, PoolId, Tokens},
};

// TODO(saruman9): will be used in the future
#[allow(dead_code)]
pub fn create_usdt_usdc_pool() -> PoolId {
	let usdt_amount = 1_000_000_000 * CurrencyId::unit::<Balance>();
	let usdc_amount = 1_000_000_000 * CurrencyId::unit::<Balance>();
	let assets = CurrencyPair::new(CurrencyId::USDT, CurrencyId::USDC);
	let amounts = vec![usdt_amount, usdc_amount];
	create_pool(assets, amounts, Permill::zero(), Permill::from_percent(50))
}

// TODO(saruman9): doesn't work, because PICA is a native token in the runtime
#[allow(dead_code)]
pub fn create_pica_ksm_pool() -> PoolId {
	let pica_amount = 1_000_000_000 * CurrencyId::unit::<Balance>();
	let ksm_amount = 1_000_000_000 * CurrencyId::unit::<Balance>();
	let assets = CurrencyPair::new(CurrencyId::PICA, CurrencyId::KSM);
	let amounts = vec![pica_amount, ksm_amount];
	create_pool(assets, amounts, Permill::zero(), Permill::from_percent(50))
}

pub fn create_layr_crowd_loan_pool() -> PoolId {
	let layr_amount = 1_000_000_000 * CurrencyId::unit::<Balance>();
	let crowd_loan_amount = 1_000_000_000 * CurrencyId::unit::<Balance>();
	let assets = CurrencyPair::new(CurrencyId::LAYR, CurrencyId::CROWD_LOAN);
	let amounts = vec![layr_amount, crowd_loan_amount];
	create_pool(assets, amounts, Permill::zero(), Permill::from_percent(50))
}

fn create_pool(
	assets: CurrencyPair<CurrencyId>,
	amounts: Vec<Balance>,
	fee: Permill,
	base_weight: Permill,
) -> PoolId {
	let base = assets.base;
	let quote = assets.quote;
	assert_ok!(Tokens::mint_into(base, &ALICE, amounts[0]));
	assert_ok!(Tokens::mint_into(quote, &ALICE, amounts[1]));
	assert_ok!(Tokens::mint_into(base, &BOB, amounts[0]));
	assert_ok!(Tokens::mint_into(quote, &BOB, amounts[1]));

	let config = PoolInitConfiguration::<AccountId, CurrencyId, BlockNumber>::ConstantProduct {
		owner: ALICE,
		pair: assets,
		fee,
		base_weight,
	};
	let pool_id = Pablo::do_create_pool(config);
	assert_ok!(pool_id);
	let pool_id = pool_id.unwrap();
	assert_ok!(<Pablo as Amm>::add_liquidity(
		&ALICE, pool_id, amounts[0], amounts[1], 0_u128, true
	));
	assert_ok!(<Pablo as Amm>::add_liquidity(&BOB, pool_id, amounts[0], amounts[1], 0_u128, true));
	pool_id
}
