use crate::mock::currency::defs::*;
use crate::mock::runtime::{
	accounts::*, AssetId, Assets, Balance, Moment, Origin, TokenizedOptions, Vault, VaultId,
};
use crate::types::*;
use composable_traits::vault::{Vault as VaultTrait, VaultConfig};
use frame_support::{assert_ok, traits::fungibles::Mutate};
use itertools::Itertools;
use proptest::{
	prelude::*,
	prop_oneof,
	strategy::{Just, Strategy},
};

use sp_runtime::Perquintill;
use std::collections::BTreeMap;

pub mod create_option;
pub mod create_vault;
// pub mod sell_option;

// ----------------------------------------------------------------------------------------------------
//		Setup VaultConfigBuilder
// ----------------------------------------------------------------------------------------------------
struct VaultConfigBuilder {
	pub asset_id: AssetId,
	pub manager: AccountId,
	// pub reserved: Perquintill,
	// pub strategies: BTreeMap<AccountId, Perquintill>,
}

impl Default for VaultConfigBuilder {
	fn default() -> Self {
		VaultConfigBuilder {
			asset_id: BTC::ID,
			manager: ADMIN,
			// reserved: Perquintill::one(),
			// strategies: BTreeMap::new(),
		}
	}
}

impl VaultConfigBuilder {
	fn build(self) -> VaultConfig<AccountId, AssetId> {
		VaultConfig {
			asset_id: self.asset_id,
			manager: self.manager,
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		}
	}

	fn asset_id(mut self, asset: AssetId) -> Self {
		self.asset_id = asset;
		self
	}

	fn manager(mut self, manager: AccountId) -> Self {
		self.manager = manager;
		self
	}
}

// ----------------------------------------------------------------------------------------------------
//		Setup VaultInitializer
// ----------------------------------------------------------------------------------------------------
pub trait VaultInitializer {
	fn initialize_vaults(self, configs: Vec<VaultConfig<AccountId, AssetId>>) -> Self;

	fn initialize_reserves(self, reserves: Vec<(AssetId, Balance)>) -> Self;

	fn initialize_vaults_with_reserves(
		self,
		configs: Vec<VaultConfig<AccountId, AssetId>>,
		reserves: Vec<(AssetId, Balance)>,
	) -> Self;
}

impl VaultInitializer for sp_io::TestExternalities {
	fn initialize_vaults(mut self, configs: Vec<VaultConfig<AccountId, AssetId>>) -> Self {
		self.execute_with(|| {
			configs.iter().for_each(|config| {
				TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), config.clone()).ok();
			});
		});

		self
	}

	fn initialize_reserves(mut self, reserves: Vec<(AssetId, Balance)>) -> Self {
		self.execute_with(|| {
			reserves.iter().for_each(|&(asset, balance)| {
				assert_ok!(<Assets as Mutate<AccountId>>::mint_into(asset, &ADMIN, balance));

				let vault_id: VaultId = Vault::token_vault(asset).unwrap();

				assert_ok!(Vault::deposit(Origin::signed(ADMIN), vault_id, balance));
			});
		});

		self
	}

	fn initialize_vaults_with_reserves(
		self,
		configs: Vec<VaultConfig<AccountId, AssetId>>,
		reserves: Vec<(AssetId, Balance)>,
	) -> Self {
		self.initialize_vaults(configs).initialize_reserves(reserves)
	}
}

// ----------------------------------------------------------------------------------------------------
//		Setup OptionsBuilder
// ----------------------------------------------------------------------------------------------------
struct OptionsConfigBuilder {
	pub base_asset_id: AssetId,
	pub quote_asset_id: AssetId,
	pub base_asset_strike_price: Balance,
	pub option_type: OptionType,
	pub exercise_type: ExerciseType,
	pub expiring_date: Moment,
	pub base_asset_amount_per_option: Balance,
	pub total_issuance_seller: Balance,
	pub total_issuance_buyer: Balance,
	pub epoch: Epoch<Moment>,
}

impl Default for OptionsConfigBuilder {
	fn default() -> Self {
		OptionsConfigBuilder {
			base_asset_id: BTC::ID,
			quote_asset_id: USDC::ID,
			base_asset_strike_price: 50000u128,
			option_type: OptionType::Call,
			exercise_type: ExerciseType::European,
			expiring_date: 1u64,
			base_asset_amount_per_option: 1u128,
			total_issuance_seller: 10u128,
			total_issuance_buyer: 10u128,
			epoch: Epoch {
				deposit: 1u64,
				purchase: 1u64,
				exercise: 1u64,
				withdraw: 1u64,
				end: 1u64,
			},
		}
	}
}

impl OptionsConfigBuilder {
	fn build(self) -> OptionConfig<AssetId, Balance, Moment> {
		OptionConfig {
			base_asset_id: self.base_asset_id,
			quote_asset_id: self.quote_asset_id,
			base_asset_strike_price: self.base_asset_strike_price,
			option_type: self.option_type,
			exercise_type: self.exercise_type,
			expiring_date: self.expiring_date,
			base_asset_amount_per_option: self.base_asset_amount_per_option,
			total_issuance_seller: self.total_issuance_seller,
			total_issuance_buyer: self.total_issuance_buyer,
			epoch: self.epoch,
		}
	}

	fn base_asset_id(mut self, base_asset_id: AssetId) -> Self {
		self.base_asset_id = base_asset_id;
		self
	}

	fn base_asset_strike_price(mut self, base_asset_strike_price: Balance) -> Self {
		self.base_asset_strike_price = base_asset_strike_price;
		self
	}
}

// ----------------------------------------------------------------------------------------------------
//		Prop Compose
// ----------------------------------------------------------------------------------------------------
#[allow(dead_code)]
pub fn pick_currency() -> impl Strategy<Value = AssetId> {
	prop_oneof![
		Just(PICA::ID),
		Just(USDC::ID),
		Just(BTC::ID),
		Just(LAYR::ID),
		Just(DOT::ID),
		Just(KSM::ID),
		Just(ETH::ID),
	]
}

#[allow(dead_code)]
pub fn pick_account() -> impl Strategy<Value = AccountId> {
	prop_oneof![Just(ALICE), Just(BOB), Just(CHARLIE), Just(DAVE), Just(EVEN),]
}

#[allow(dead_code)]
const MINIMUM_RESERVE: Balance = 1_000;

#[allow(dead_code)]
const MAXIMUM_RESERVE: Balance = 1_000_000_000;

#[allow(dead_code)]
const MINIMUM_STRIKE_PRICE: Balance = 1;

#[allow(dead_code)]
const MAXIMUM_STRIKE_PRICE: Balance = 1_000_000;

#[allow(dead_code)]
const TOTAL_NUM_OF_ASSETS: usize = 7;

#[allow(dead_code)]
const TOTAL_NUM_OF_ACCOUNTS: usize = 5;

#[allow(dead_code)]
const NUMBER_OF_PROPTEST_CASES: u32 =
	3u32 * TOTAL_NUM_OF_ASSETS as u32 * TOTAL_NUM_OF_ACCOUNTS as u32;

prop_compose! {
	fn generate_accounts()(
		accounts in prop::collection::vec(pick_account(), 1..=TOTAL_NUM_OF_ACCOUNTS),
	) -> Vec<AccountId>{
		accounts
   }
}

prop_compose! {
	fn generate_assets()(
		assets in prop::collection::vec(pick_currency(), 1..=TOTAL_NUM_OF_ASSETS),
	) -> Vec<AssetId>{
		assets
   }
}

prop_compose! {
	fn generate_balances()(
		balances in prop::collection::vec(MINIMUM_RESERVE..MAXIMUM_RESERVE, 1..=TOTAL_NUM_OF_ASSETS),
	) -> Vec<Balance>{
		balances
   }
}

prop_compose! {
	fn generate_prices()(
		prices in prop::collection::vec(MINIMUM_STRIKE_PRICE..MAXIMUM_STRIKE_PRICE, 1..=TOTAL_NUM_OF_ASSETS),
	) -> Vec<Balance>{
		prices
   }
}

prop_compose! {
	fn generate_markets()(assets in generate_assets(), prices in generate_prices()) -> Vec<(AssetId, Balance)>{
		assets.into_iter().unique().zip(prices.into_iter()).collect()
	}
}

prop_compose! {
	fn generate_blockchain_state()(
		accounts in generate_accounts(),
		assets in generate_assets(),
		balances in generate_balances(),
		asset_prices in generate_prices()
	) -> Vec<(AccountId, AssetId, Balance, Balance)>{
		accounts.into_iter()
			.zip(assets.into_iter())
			.unique()
			.zip(balances.into_iter())
			.unique()
			.zip(asset_prices.into_iter())
			.map(|(((account, asset), balance), asset_price)| (account, asset, balance, asset_price))
			.collect()
   }
}
