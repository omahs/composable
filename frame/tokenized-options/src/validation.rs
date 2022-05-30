use crate::pallet::{
	AssetToVault, BalanceOf, Config, OptionConfigOf, OptionHashToOptionId, OracleOf, Pallet,
	VaultConfigOf,
};

use composable_support::validation::Validate;

use composable_traits::oracle::Oracle;
use core::marker::PhantomData;
use frame_support::traits::Time;
use sp_runtime::traits::Zero;
use sp_std::cmp::max;

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
			input.quote_asset_id,
			input.base_asset_strike_price,
			input.quote_asset_strike_price,
			input.option_type,
			input.expiring_date,
			input.exercise_type,
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

// -----------------------------------------------------------------------------------------------
//		ValidateOptionAttributes
// -----------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct ValidateOptionAttributes<T> {
	_marker: PhantomData<T>,
}

impl<T: Config> Validate<OptionConfigOf<T>, ValidateOptionAttributes<T>>
	for ValidateOptionAttributes<T>
{
	fn validate(input: OptionConfigOf<T>) -> Result<OptionConfigOf<T>, &'static str> {
		if input.total_issuance_seller != BalanceOf::<T>::zero()
			|| input.total_issuance_buyer != BalanceOf::<T>::zero()
		{
			return Err("ValidateOptionAttributes");
		}

		let start = max(<T as Config>::Time::now(), input.epoch.deposit);

		if start >= input.epoch.purchase
			|| input.epoch.purchase >= input.epoch.exercise
			|| input.epoch.exercise >= input.epoch.withdraw
			|| input.epoch.withdraw >= input.epoch.end
		{
			return Err("ValidateOptionAttributes");
		}

		Ok(input)
	}
}
