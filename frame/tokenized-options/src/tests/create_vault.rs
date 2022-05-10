use crate::mock::currency::defs::*;
use crate::mock::runtime::{
	accounts::*, Event, ExtBuilder, MockRuntime, Origin, System, TokenizedOptions, VaultId,
};
use crate::pallet::{self, AssetToVault, Error};
use crate::tests::*;
use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use frame_system::ensure_signed;

use frame_support::{assert_noop, assert_ok};

// Simulate exstrinsic call `create_asset_vault`, but returning values
fn trait_create_asset_vault(
	_origin: Origin,
	vault_config: VaultConfig<AccountId, AssetId>,
) -> VaultId {
	let _account_id = ensure_signed(_origin).unwrap();

	let vault_id =
		<TokenizedOptions as TokenizedOptionsTrait>::create_asset_vault(vault_config.clone())
			.unwrap();

	TokenizedOptions::deposit_event(pallet::Event::CreatedAssetVault {
		vault_id,
		asset_id: vault_config.asset_id,
	});

	vault_id
}

// ----------------------------------------------------------------------------------------------------
//		Create Vault Tests
// ----------------------------------------------------------------------------------------------------
#[test]
fn test_create_vault_and_emit_event() {
	ExtBuilder::default().build().execute_with(|| {
		// Get default vault config
		let vault_config = VaultConfigBuilder::default().build();

		// Create vault
		let vault_id: VaultId =
			trait_create_asset_vault(Origin::signed(ADMIN), vault_config.clone());

		// Check vault has been created
		assert!(AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id,
			asset_id: vault_config.asset_id,
		}));
	});
}

#[test]
fn test_create_same_vault_and_emit_error() {
	ExtBuilder::default().build().execute_with(|| {
		// Get default vault config
		let vault_config = VaultConfigBuilder::default().build();

		let vault_id: VaultId =
			trait_create_asset_vault(Origin::signed(ADMIN), vault_config.clone());

		// Check vault has been created
		assert!(AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));

		// Check event is emitted correctly
		System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
			vault_id,
			asset_id: vault_config.asset_id,
		}));

		// Create same vault again and check error is raised
		assert_noop!(
			TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), vault_config.clone()),
			Error::<MockRuntime>::AssetVaultAlreadyExists
		);
	});
}

proptest! {
	#![proptest_config(ProptestConfig::with_cases(10))]
	#[test]
	fn proptest_create_vault(assets in generate_assets()) {
		ExtBuilder::default().build().execute_with(|| {
			assets.iter().for_each(|&asset| {
				// Get vault config with custom asset
				let vault_config = VaultConfigBuilder::default().asset_id(asset).build();

				// Check if vault has not been already created and create it or raise error
				if !AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id) {
					let vault_id: VaultId =
					trait_create_asset_vault(Origin::signed(ADMIN), vault_config.clone());

					// Check vault has been created
					assert!(AssetToVault::<MockRuntime>::contains_key(vault_config.asset_id));

					// Check event is emitted correctly
					System::assert_last_event(Event::TokenizedOptions(pallet::Event::CreatedAssetVault {
						vault_id,
						asset_id: vault_config.asset_id,
					}
				));
				} else {
					assert_noop!(
						TokenizedOptions::create_asset_vault(Origin::signed(ADMIN), vault_config.clone()),
						Error::<MockRuntime>::AssetVaultAlreadyExists
					);
				}
			});
		});
	}
}
