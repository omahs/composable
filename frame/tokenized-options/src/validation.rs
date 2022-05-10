use crate::pallet::{AssetToVault, Config, OptionConfigOf, VaultConfigOf};

use composable_support::validation::Validate;
// use composable_traits::tokenized_options::*;

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
	fn validate(input: VaultConfigOf<T>) -> Result<VaultConfigOf<T>, &'static str> {
		if AssetToVault::<T>::contains_key(input.asset_id) {
			return Err("Vault Already Exists");
		}

		Ok(input)
	}
}

// -----------------------------------------------------------------------------------------------
//		ValidateOptionDoesNotExist
// -----------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct ValidateOptionDoesNotExist<T> {
	_marker: PhantomData<T>,
}

impl<T: Config> Validate<OptionConfigOf<T>, ValidateOptionDoesNotExist<T>>
	for ValidateOptionDoesNotExist<T>
{
	fn validate(input: OptionConfigOf<T>) -> Result<OptionConfigOf<T>, &'static str> {
		// Need to implement how to check if Option already exists
		// False check right now just for code to compile
		if input.base_asset_id == input.quote_asset_id {
			return Err("Same base and quote assets!");
		}

		Ok(input)
	}
}
