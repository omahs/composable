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

// pub mod create_option;
pub mod create_vault;
// pub mod sell_option;

// ----------------------------------------------------------------------------------------------------
//		VaultConfigBuilder
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
//		VaultInitializer
// ----------------------------------------------------------------------------------------------------
pub trait VaultInitializer {
	fn initialize_vaults(self, configs: Vec<VaultConfig<AccountId, AssetId>>) -> Self;

	fn initialize_deposits(self, deposits: Vec<(AssetId, Balance)>) -> Self;

	fn initialize_vaults_with_deposits(
		self,
		vault_configs: Vec<VaultConfig<AccountId, AssetId>>,
		deposits: Vec<(AssetId, Balance)>,
	) -> Self;
}

impl VaultInitializer for sp_io::TestExternalities {
	fn initialize_vaults(mut self, vault_configs: Vec<VaultConfig<AccountId, AssetId>>) -> Self {
		self.execute_with(|| {
			vault_configs.iter().for_each(|config| {
				TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), config.clone()).ok();
			});
		});

		self
	}

	fn initialize_deposits(mut self, deposits: Vec<(AssetId, Balance)>) -> Self {
		self.execute_with(|| {
			deposits.iter().for_each(|&(asset, balance)| {
				assert_ok!(<Assets as Mutate<AccountId>>::mint_into(asset, &ADMIN, balance));

				let vault_id: VaultId = Vault::token_vault(asset).unwrap();

				assert_ok!(Vault::deposit(Origin::signed(ADMIN), vault_id, balance));
			});
		});

		self
	}

	fn initialize_vaults_with_deposits(
		self,
		vault_configs: Vec<VaultConfig<AccountId, AssetId>>,
		deposits: Vec<(AssetId, Balance)>,
	) -> Self {
		self.initialize_vaults(vault_configs).initialize_deposits(deposits)
	}
}

// ----------------------------------------------------------------------------------------------------
//		OptionsConfigBuilder
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
//		OptionInitializer
// ----------------------------------------------------------------------------------------------------

pub trait OptionInitializer {
	fn initialize_options(
		self,
		option_configs: Vec<OptionConfig<AssetId, Balance, Moment>>,
	) -> Self;
}

// ----------------------------------------------------------------------------------------------------
//		Prop Compose
// ----------------------------------------------------------------------------------------------------

pub const VEC_SIZE: usize = 10;

pub fn pick_asset() -> impl Strategy<Value = AssetId> {
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

pub fn pick_account() -> impl Strategy<Value = AccountId> {
	prop_oneof![Just(ALICE), Just(BOB), Just(CHARLIE), Just(DAVE), Just(EVEN),]
}

prop_compose! {
	fn prop_random_account()
		(x in pick_account()) -> AccountId {
			x
		}
}

prop_compose! {
	fn prop_random_asset()
		(x in pick_asset()) -> AssetId {
			x
		}
}

prop_compose! {
	fn prop_random_balance()
		(x in 0..Balance::MAX) -> Balance {
			x
		}
}

prop_compose! {
	fn prop_random_account_vec()(
		accounts in prop::collection::vec(pick_account(), 1..=VEC_SIZE),
	) -> Vec<AccountId>{
		accounts
   }
}

prop_compose! {
	fn prop_random_asset_vec()(
		assets in prop::collection::vec(pick_asset(), 1..=VEC_SIZE),
	) -> Vec<AccountId>{
		assets
   }
}

prop_compose! {
	fn prop_random_balance_vec()(
		balances in prop::collection::vec(prop_random_balance(), 1..=VEC_SIZE),
	) -> Vec<AccountId>{
		balances
   }
}

prop_compose! {
	fn prop_random_market()(account in prop_random_account(), asset in prop_random_asset(), amount in prop_random_balance(), price in prop_random_balance()) -> (AccountId, AssetId, Balance, Balance){
		(account, asset, amount, price)
	}
}

prop_compose! {
	fn prop_random_market_vec()(
		accounts in prop_random_account_vec(),
		assets in prop_random_asset_vec(),
		balances in prop_random_balance_vec(),
		asset_prices in prop_random_balance_vec()
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
