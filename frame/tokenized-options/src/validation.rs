use crate::pallet::{
	AssetToVault, Config, OptionConfigOf, OptionHashToOptionId, OracleOf, Pallet, VaultConfigOf,
};

use composable_support::validation::Validate;

use composable_traits::oracle::Oracle;

use core::marker::PhantomData;

// -----------------------------------------------------------------------------------------------
//		ValidateVaultDoesNotExist
// -----------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct ValidateVaultDoesNotExist<T> {
	_marker: PhantomData<T>,
}

impl<T: Config> Validate<VaultConfigOf<T>, ValidateVaultDoesNotExist<T>>
	for ValidateVaultDoesNotExist<T>
{
	fn validate(vault_config: VaultConfigOf<T>) -> Result<VaultConfigOf<T>, &'static str> {
		if AssetToVault::<T>::contains_key(vault_config.asset_id) {
			return Err("ValidateVaultDoesNotExist");
		}

		Ok(vault_config)
	}
}

// -----------------------------------------------------------------------------------------------
//		ValidateAssetIsSupported
// -----------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct ValidateAssetIsSupported<T> {
	_marker: PhantomData<T>,
}

impl<T: Config> Validate<VaultConfigOf<T>, ValidateAssetIsSupported<T>>
	for ValidateAssetIsSupported<T>
{
	fn validate(vault_config: VaultConfigOf<T>) -> Result<VaultConfigOf<T>, &'static str> {
		match OracleOf::<T>::is_supported(vault_config.asset_id) {
			Ok(_) => Ok(vault_config),
			Err(_) => return Err("ValidateAssetIsSupported"),
		}
	}
}

// // -----------------------------------------------------------------------------------------------
// //		ValidateOptionDoesNotExist
// // -----------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct ValidateOptionDoesNotExist<T> {
	_marker: PhantomData<T>,
}

impl<T: Config> Validate<OptionConfigOf<T>, ValidateOptionDoesNotExist<T>>
	for ValidateOptionDoesNotExist<T>
{
	fn validate(input: OptionConfigOf<T>) -> Result<OptionConfigOf<T>, &'static str> {
		let hash = Pallet::<T>::generate_id(
			input.base_asset_id,
			input.base_asset_strike_price,
			input.option_type,
			input.expiring_date,
		);
		if OptionHashToOptionId::<T>::contains_key(hash) {
			return Err("ValidateOptionDoesNotExist");
		}

		Ok(input)
	}
}

// -----------------------------------------------------------------------------------------------
//		ValidateOptionAssetVaultsExist
// -----------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct ValidateOptionAssetVaultsExist<T> {
	_marker: PhantomData<T>,
}

impl<T: Config> Validate<OptionConfigOf<T>, ValidateOptionAssetVaultsExist<T>>
	for ValidateOptionAssetVaultsExist<T>
{
	fn validate(input: OptionConfigOf<T>) -> Result<OptionConfigOf<T>, &'static str> {
		if !(AssetToVault::<T>::contains_key(input.base_asset_id)
			&& AssetToVault::<T>::contains_key(input.quote_asset_id)
			&& input.base_asset_id != input.quote_asset_id)
		{
			return Err("ValidateOptionAssetVaultsExist");
		}

		Ok(input)
	}
}
