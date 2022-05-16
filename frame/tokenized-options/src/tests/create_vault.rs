use crate::mock::currency::defs::*;
use crate::mock::runtime::{
	accounts::*, Event, ExtBuilder, MockRuntime, Origin, System, TokenizedOptions, VaultId,
};
use crate::pallet::{self, AssetToVault, Error};
use crate::tests::*;
use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use frame_system::ensure_signed;

use frame_support::{assert_err, assert_noop};

// ----------------------------------------------------------------------------------------------------
//		Create Vault Tests
// ----------------------------------------------------------------------------------------------------

/// Create BTC vault and check if vault_id is correctly saved and event emitted
#[test]
fn test_create_vault_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		// Get default vault config
		let vault_config = VaultConfigBuilder::default().build();

		// Check that the vault has not already been created
		assert!(!AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));

		// Create vault
		let vault_id = trait_create_asset_vault(Origin::signed(ADMIN), vault_config.clone())
			.expect("Error creating vault");

		// Check vault has been created
		assert!(AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id,
			asset_id: vault_config.asset_id,
		}));
	});
}

/// Create BTC vault using extrinsic and check if vault_id is correctly saved and event emitted
#[test]
fn test_create_vault_and_emit_event_ext() {
	ExtBuilder::default().build().execute_with(|| {
		let vault_config = VaultConfigBuilder::default().build();

		assert!(!AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));

		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			vault_config.clone()
		));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id: 1u64,
			asset_id: vault_config.asset_id,
		}));
	});
}

/// Create BTC vault correctly using exstrinsic and try to create it again, check if error is raised and storage not changed
#[test]
fn test_create_same_vault_and_emit_error_ext() {
	ExtBuilder::default().build().execute_with(|| {
		let vault_config = VaultConfigBuilder::default().build();

		assert!(!AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));

		assert_ok!(TokenizedOptions::create_asset_vault(
			Origin::signed(ADMIN),
			vault_config.clone()
		));

		assert!(AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));

		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id: 1u64,
			asset_id: vault_config.asset_id,
		}));

		assert_noop!(
			TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), vault_config.clone()),
			Error::<MockRuntime>::AssetVaultAlreadyExists
		);
	});
}

// TODO: try to create vault with no-admin account and check error raised

proptest! {
	#![proptest_config(ProptestConfig::with_cases(50))]
	#[test]
	fn proptest_create_vault_ext(assets in prop_random_asset_vec()) {
		ExtBuilder::default().build().execute_with(|| {
			assets.iter().for_each(|&asset| {
				let vault_config = VaultConfigBuilder::default().asset_id(asset).build();

				if !AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id) {
					assert_ok!(TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), vault_config.clone()));
					assert!(AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));
				} else {
					assert_noop!(
						TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), vault_config),
						Error::<MockRuntime>::AssetVaultAlreadyExists
					);
				}
			});
		});
	}

}
