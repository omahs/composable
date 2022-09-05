use composable_traits::{
	instrumental::{AccessRights, InstrumentalProtocolStrategy},
	vault::{CapabilityVault, StrategicVault, Vault as VaultTrait},
};
use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect};
use primitives::currency::CurrencyId;
use sp_runtime::Perquintill;

use crate::mock::{
	account_id::ADMIN,
	helpers::{create_pool, create_vault},
	runtime::{
		Assets, Balance, Event, ExtBuilder, Instrumental, MockRuntime, Origin, PabloStrategy,
		System, Vault, VaultId, MAX_ASSOCIATED_VAULTS,
	},
};
#[allow(unused_imports)]
use crate::{pallet, pallet::Error};

// -------------------------------------------------------------------------------------------------
//                                          Associate Vault
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod associate_vault {
	use super::*;

	#[test]
	fn add_an_associated_vault() {
		ExtBuilder::default().build().execute_with(|| {
			let vault_id: VaultId = 1;

			assert_ok!(PabloStrategy::associate_vault(&vault_id));
		});
	}

	#[test]
	fn adding_an_associated_vault_twice_throws_an_error() {
		ExtBuilder::default().build().execute_with(|| {
			let vault_id: VaultId = 1;

			assert_ok!(PabloStrategy::associate_vault(&vault_id));
			assert_noop!(
				PabloStrategy::associate_vault(&vault_id),
				Error::<MockRuntime>::VaultAlreadyAssociated
			);
		});
	}

	#[test]
	fn associating_too_many_vaults_throws_an_error() {
		ExtBuilder::default().build().execute_with(|| {
			for vault_id in 0..MAX_ASSOCIATED_VAULTS {
				assert_ok!(PabloStrategy::associate_vault(&(vault_id as VaultId)));
			}

			let vault_id = MAX_ASSOCIATED_VAULTS as VaultId;
			assert_noop!(
				PabloStrategy::associate_vault(&vault_id),
				Error::<MockRuntime>::TooManyAssociatedStrategies
			);
		});
	}
}

// -------------------------------------------------------------------------------------------------
//                                             Rebalance
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod rebalance {

	use super::*;

	#[test]
	fn rebalance_emits_event() {
		ExtBuilder::default().build().execute_with(|| {
			System::set_block_number(1);
			let base_asset = CurrencyId::LAYR;
			let quote_asset = CurrencyId::CROWD_LOAN;
			let amount = 1_000_000_000 * CurrencyId::unit::<Balance>();

			// Create Vault (LAYR)
			let vault_id = create_vault(base_asset, None);

			// Create Pool (LAYR/CROWD_LOAN)
			let pool_id = create_pool(base_asset, amount, quote_asset, amount, None, None);
			pallet::AdminAccountIds::<MockRuntime>::insert(ADMIN, AccessRights::Full);
			assert_ok!(PabloStrategy::set_pool_id_for_asset(
				Origin::signed(ADMIN),
				base_asset,
				pool_id
			));

			assert_ok!(PabloStrategy::associate_vault(&vault_id));

			assert_ok!(PabloStrategy::rebalance());

			System::assert_last_event(Event::PabloStrategy(pallet::Event::RebalancedVault {
				vault_id,
			}));
		});
	}

	#[test]
	fn funds_availability_withdrawable() {
		let asset = CurrencyId::LAYR;
		let amount = 100 * CurrencyId::unit::<Balance>();

		ExtBuilder::default()
			.initialize_balance(ADMIN, asset, amount)
			.build()
			.execute_with(|| {
				System::set_block_number(1);

				let vault_id = create_vault(asset, Perquintill::from_percent(10));
				let vault_account = Vault::account_id(&vault_id);

				let pool_id = create_pool(asset, None, None, None, None, None);
				pallet::AdminAccountIds::<MockRuntime>::insert(ADMIN, AccessRights::Full);
				assert_ok!(PabloStrategy::set_pool_id_for_asset(
					Origin::signed(ADMIN),
					asset,
					pool_id
				));

				assert_ok!(<PabloStrategy as InstrumentalProtocolStrategy>::associate_vault(
					&vault_id
				));

				dbg!(Assets::balance(asset, &vault_account));

				dbg!(<Vault as StrategicVault>::available_funds(
					&vault_id,
					&PabloStrategy::account_id(),
				));
				assert_ok!(Instrumental::add_liquidity(
					Origin::signed(ADMIN),
					asset,
					90 * CurrencyId::unit::<Balance>()
				));

				dbg!(Assets::balance(asset, &vault_account));

				dbg!(<Vault as StrategicVault>::available_funds(
					&vault_id,
					&PabloStrategy::account_id(),
				));
				// Tokens::mint_into(asset, &ADMIN, 100 * CurrencyId::unit::<Balance>());
				// Vault::deposit(
				// 	Origin::signed(ADMIN),
				// 	vault_id,
				// 	100 * CurrencyId::unit::<Balance>(),
				// );
				// dbg!(Tokens::balance(asset, &ADMIN));
				// dbg!(<Vault as StrategicVault>::available_funds(
				// 	&vault_id,
				// 	&PabloStrategy::account_id()
				// ));

				assert_ok!(PabloStrategy::rebalance());
				System::assert_last_event(Event::PabloStrategy(pallet::Event::RebalancedVault {
					vault_id,
				}));

				dbg!(<Vault as StrategicVault>::available_funds(
					&vault_id,
					&PabloStrategy::account_id()
				));
			})
	}

	#[test]
	fn funds_availability_depositable() {
		let base_asset = CurrencyId::LAYR;
		let amount = 100 * CurrencyId::unit::<Balance>();

		ExtBuilder::default()
			.initialize_balance(ADMIN, base_asset, amount)
			.build()
			.execute_with(|| {
				System::set_block_number(1);

				let base_vault_id = create_vault(base_asset, Perquintill::from_percent(10));
				let base_vault_account = Vault::account_id(&base_vault_id);

				let pool_id = create_pool(base_asset, None, None, None, None, None);
				pallet::AdminAccountIds::<MockRuntime>::insert(ADMIN, AccessRights::Full);
				assert_ok!(PabloStrategy::set_pool_id_for_asset(
					Origin::signed(ADMIN),
					base_asset,
					pool_id
				));

				assert_ok!(<PabloStrategy as InstrumentalProtocolStrategy>::associate_vault(
					&base_vault_id
				));
			})
	}

	#[test]
	fn funds_availability_must_liquidate_without_depositing() {
		let asset = CurrencyId::LAYR;

		ExtBuilder::default().build().execute_with(|| {
			System::set_block_number(1);

			let vault_id = create_vault(asset, Perquintill::from_percent(10));
			let vault_account = Vault::account_id(&vault_id);

			let pool_id = create_pool(asset, None, None, None, None, None);
			pallet::AdminAccountIds::<MockRuntime>::insert(ADMIN, AccessRights::Full);
			assert_ok!(PabloStrategy::set_pool_id_for_asset(Origin::signed(ADMIN), asset, pool_id));

			assert_ok!(<PabloStrategy as InstrumentalProtocolStrategy>::associate_vault(&vault_id));

			assert_ok!(Vault::stop(&vault_id));

			assert_ok!(PabloStrategy::rebalance());

			System::assert_last_event(Event::PabloStrategy(pallet::Event::RebalancedVault {
				vault_id,
			}));
		})
	}

	#[test]
	fn funds_availability_must_liquidate() {
		todo!()
	}

	#[test]
	fn funds_availability_none() {
		let asset = CurrencyId::LAYR;

		ExtBuilder::default().build().execute_with(|| {
			System::set_block_number(1);

			let vault_id = create_vault(asset, None);
			let pool_id = create_pool(asset, None, None, None, None, None);
			pallet::AdminAccountIds::<MockRuntime>::insert(ADMIN, AccessRights::Full);
			assert_ok!(PabloStrategy::set_pool_id_for_asset(Origin::signed(ADMIN), asset, pool_id));

			assert_ok!(<PabloStrategy as InstrumentalProtocolStrategy>::associate_vault(&vault_id));
			assert_ok!(PabloStrategy::rebalance());
		})
	}
}
