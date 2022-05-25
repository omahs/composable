use crate::mock::accounts::*;
use crate::mock::assets::*;
use crate::mock::runtime::{
	get_oracle_price, set_oracle_price, Assets, Balance, MockRuntime, Moment, Origin,
	TokenizedOptions, Vault, VaultId,
};
use crate::pallet::{self, AssetToVault, Error, OptionIdToOption};
use crate::types::*;
use composable_traits::{
	tokenized_options::TokenizedOptions as TokenizedOptionsTrait,
	vault::{Vault as VaultTrait, VaultConfig},
};
use frame_system::ensure_signed;

use frame_support::{assert_ok, traits::fungibles::Mutate};
use itertools::Itertools;
use proptest::{
	prelude::*,
	prop_oneof,
	strategy::{Just, Strategy},
};
use sp_runtime::DispatchError;
use sp_runtime::Perquintill;
use std::collections::BTreeMap;

pub mod create_option;
pub mod create_vault;
pub mod sell_option;

// ----------------------------------------------------------------------------------------------------
//		VaultConfigBuilder
// ----------------------------------------------------------------------------------------------------
struct VaultConfigBuilder {
	pub asset_id: AssetId,
	pub manager: AccountId,
	pub reserved: Perquintill,
	pub strategies: BTreeMap<AccountId, Perquintill>,
}

impl Default for VaultConfigBuilder {
	fn default() -> Self {
		VaultConfigBuilder {
			asset_id: BTC,
			manager: ADMIN,
			reserved: Perquintill::one(),
			strategies: BTreeMap::new(),
		}
	}
}

impl VaultConfigBuilder {
	fn build(self) -> VaultConfig<AccountId, AssetId> {
		VaultConfig {
			asset_id: self.asset_id,
			manager: self.manager,
			reserved: self.reserved,
			strategies: self.strategies,
		}
	}

	fn asset_id(mut self, asset: AssetId) -> Self {
		self.asset_id = asset;
		self
	}
}

// ----------------------------------------------------------------------------------------------------
//		VaultInitializer
// ----------------------------------------------------------------------------------------------------
pub trait VaultInitializer {
	fn initialize_vaults(self, configs: Vec<VaultConfig<AccountId, AssetId>>) -> Self;

	fn initialize_all_vaults(self) -> Self;

	fn initialize_oracle_prices(self) -> Self;

	fn initialize_deposits(self, deposits: Vec<(AssetId, Balance)>) -> Self;

	fn initialize_vaults_with_deposits(
		self,
		vault_configs: Vec<VaultConfig<AccountId, AssetId>>,
		deposits: Vec<(AssetId, Balance)>,
	) -> Self;
}

impl VaultInitializer for sp_io::TestExternalities {
	fn initialize_oracle_prices(mut self) -> Self {
		let assets_prices: Vec<(AssetId, Balance)> =
			Vec::from([(USDC, 1), (BTC, 50_000), (DOT, 100), (PICA, 1_000), (LAYR, 10_000)]);

		self.execute_with(|| {
			assets_prices.iter().for_each(|&(asset, price)| {
				set_oracle_price(asset, price * 10u128.pow(12));
			});
		});

		self
	}

	fn initialize_all_vaults(mut self) -> Self {
		let assets = Vec::from([PICA, BTC, USDC, LAYR]);

		self.execute_with(|| {
			assets.iter().for_each(|&asset| {
				let config = VaultConfigBuilder::default().asset_id(asset).build();
				TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), config.clone()).ok();
			});
		});

		self
	}

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
			base_asset_id: BTC,
			quote_asset_id: USDC,
			base_asset_strike_price: 50000u128 * 10u128.pow(12),
			option_type: OptionType::Call,
			exercise_type: ExerciseType::European,
			expiring_date: 1u64,
			base_asset_amount_per_option: 1u128 * 10u128.pow(12),
			total_issuance_seller: 0u128,
			total_issuance_buyer: 0u128,
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

	fn option_type(mut self, option_type: OptionType) -> Self {
		self.option_type = option_type;
		self
	}

	fn expiring_date(mut self, expiring_date: Moment) -> Self {
		self.expiring_date = expiring_date;
		self
	}

	fn base_asset_amount_per_option(mut self, base_asset_amount_per_option: Balance) -> Self {
		self.base_asset_amount_per_option = base_asset_amount_per_option;
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

	fn initialize_all_options(self) -> Self;
}

impl OptionInitializer for sp_io::TestExternalities {
	fn initialize_options(
		mut self,
		option_configs: Vec<OptionConfig<AssetId, Balance, Moment>>,
	) -> Self {
		self.execute_with(|| {
			option_configs.iter().for_each(|config| {
				TokenizedOptions::create_option(Origin::signed(ADMIN), config.clone()).ok();
			});
		});

		self
	}

	fn initialize_all_options(mut self) -> Self {
		let assets: Vec<AssetId> = Vec::from([BTC, DOT, PICA, LAYR]);

		assets.iter().for_each(|&asset| {
			self.execute_with(|| {
				let price = get_oracle_price(asset, 1);

				let config = OptionsConfigBuilder::default()
					.base_asset_id(asset)
					.base_asset_strike_price(price)
					.build();

				let price2 = price.checked_add(price / 10).unwrap();
				let config2 = OptionsConfigBuilder::default()
					.base_asset_id(asset)
					.base_asset_strike_price(price2)
					.build();

				let price3 = price.checked_sub(price / 10).unwrap();
				let config3 = OptionsConfigBuilder::default()
					.base_asset_id(asset)
					.base_asset_strike_price(price3)
					.build();

				TokenizedOptions::create_option(Origin::signed(ADMIN), config.clone()).ok();
				TokenizedOptions::create_option(Origin::signed(ADMIN), config2.clone()).ok();
				TokenizedOptions::create_option(Origin::signed(ADMIN), config3.clone()).ok();
			});
		});

		self
	}
}

// ----------------------------------------------------------------------------------------------------
//		Prop Compose
// ----------------------------------------------------------------------------------------------------

pub const VEC_SIZE: usize = 10;

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
	fn prop_random_account_vec()(
		accounts in prop::collection::vec(pick_account(), VEC_SIZE),
	) -> Vec<AccountId>{
		accounts
   }
}

pub fn pick_asset() -> impl Strategy<Value = AssetId> {
	prop_oneof![Just(PICA), Just(BTC), Just(LAYR), Just(DOT),]
}

prop_compose! {
	fn prop_random_asset()
		(x in pick_asset()) -> AssetId {
			x
		}
}

prop_compose! {
	fn prop_random_asset_vec()(
		assets in prop::collection::vec(pick_asset(), VEC_SIZE),
	) -> Vec<AssetId>{
		assets
   }
}

prop_compose! {
	fn prop_random_balance()
		(x in 0..Balance::MAX) -> Balance {
			x
		}
}

prop_compose! {
	fn prop_random_balance_vec()(
		balances in prop::collection::vec(prop_random_balance(), VEC_SIZE),
	) -> Vec<Balance>{
		balances
   }
}

prop_compose! {
	fn prop_random_initial_balances_vec()(
		accounts in prop_random_account_vec(),
		assets in prop_random_asset_vec(),
		balances in prop_random_balance_vec(),
	) -> Vec<(AccountId, AssetId, Balance)>{
		accounts.into_iter()
			.zip(assets.into_iter())
			.unique()
			.zip(balances.into_iter())
			.map(|((account, asset), balance)| (account, asset, balance))
			.collect()
   }
}

prop_compose! {
	fn prop_random_option_config()(base_asset_id in prop_random_asset(), base_asset_strike_price in prop_random_balance()) -> (AssetId, Balance){
		(base_asset_id, base_asset_strike_price)
	}
}

prop_compose! {
	fn prop_random_option_config_vec()(
		base_asset_ids in prop_random_asset_vec(),
		base_asset_strike_prices in prop_random_balance_vec(),
	) -> Vec<(AssetId, Balance)>{
			base_asset_ids.into_iter()
			.zip(base_asset_strike_prices.into_iter())
			.collect()
   }
}

prop_compose! {
	fn prop_random_option_amount(limit: u128)(
		s in 1u128..limit
	) -> u128 {
		s
	}
}

prop_compose! {
	fn prop_random_option_amount_vec()(
		amounts in prop::collection::vec(prop_random_option_amount(10u128), VEC_SIZE),
	) -> Vec<Balance>{
		amounts
   }
}

prop_compose! {
	fn prop_rng()(
		rng in any::<usize>()
	) -> usize {
		rng
	}
}

prop_compose! {
	fn prop_rng_vec()(
		rngs in prop::collection::vec(prop_rng(), VEC_SIZE),
	) -> Vec<usize>{
		rngs
   }
}

// ----------------------------------------------------------------------------------------------------
//		Extrinsic function simulators
// ----------------------------------------------------------------------------------------------------

// Simulate exstrinsic call `create_asset_vault`, but returning values
pub fn trait_create_asset_vault(
	_origin: Origin,
	vault_config: VaultConfig<AccountId, AssetId>,
) -> Result<VaultId, DispatchError> {
	let _account_id = ensure_signed(_origin).unwrap();

	let vault_id =
		<TokenizedOptions as TokenizedOptionsTrait>::create_asset_vault(vault_config.clone())?;

	Ok(vault_id)
}

// Simulate exstrinsic call `create_option`, but returning values
pub fn trait_create_option(
	origin: Origin,
	option_config: OptionConfig<AssetId, Balance, Moment>,
) -> Result<AssetId, DispatchError> {
	let _account_id = ensure_signed(origin).unwrap();

	let option_id =
		<TokenizedOptions as TokenizedOptionsTrait>::create_option(option_config.clone())?;

	Ok(option_id)
}
