use frame_support::pallet_prelude::*;
#[allow(unused_variables)]

pub trait TokenizedOptions {
	type AccountId;
	type Balance;
	type Moment;
	type OptionId;
	type VaultId;
	type OptionConfig;
	type VaultConfig;

	fn create_asset_vault(config: Self::VaultConfig) -> Result<Self::VaultId, DispatchError>;

	fn create_option(option_config: Self::OptionConfig) -> Result<Self::OptionId, DispatchError>;

	fn sell_option(
		from: &Self::AccountId,
		option_amount: Self::Balance,
		option_id: Self::OptionId,
	) -> Result<(), DispatchError>;

	fn delete_sell_option(
		from: &Self::AccountId,
		option_amount: Self::Balance,
		option_id: Self::OptionId,
	) -> Result<(), DispatchError>;

	fn buy_option(
		from: &Self::AccountId,
		option_amount: Self::Balance,
		option: Self::OptionId,
	) -> Result<(), DispatchError>;

	fn settle_options() -> Result<(), DispatchError>;

	fn exercise_option(
		from: &Self::AccountId,
		option_amount: Self::Balance,
		option: Self::OptionId,
	) -> Result<(), DispatchError>;

	fn withdraw_collateral(
		from: &Self::AccountId,
		option: Self::OptionId,
	) -> Result<(), DispatchError>;
}
