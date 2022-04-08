use crate as pallet_tokenized_options;
use crate::currency::{defs::*, CurrencyId};
use accounts::*;
use composable_traits::{defi::DeFiComposableConfig, governance::SignedRawOrigin};
use frame_support::{ord_parameter_types, parameter_types, traits::Everything, PalletId};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::{parameter_type_with_key, GetByKey};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{ConvertInto, IdentityLookup},
};

pub type BlockNumber = u64;
pub type AssetId = u128;
pub type Balance = u128;
pub type VaultId = u64;
pub type Amount = i128;

pub mod accounts {
	use hex_literal::hex;
	use sp_core::sr25519::{Public, Signature};
	use sp_runtime::traits::{IdentifyAccount, Verify};
	pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	pub static ADMIN: Public =
		Public(hex!("0000000000000000000000000000000000000000000000000000000000000000"));
	pub static ALICE: Public =
		Public(hex!("0000000000000000000000000000000000000000000000000000000000000001"));
	pub static BOB: Public =
		Public(hex!("0000000000000000000000000000000000000000000000000000000000000002"));
	pub static CHARLIE: Public =
		Public(hex!("0000000000000000000000000000000000000000000000000000000000000003"));
	pub static DAVE: Public =
		Public(hex!("0000000000000000000000000000000000000000000000000000000000000004"));
	pub static EVEN: Public =
		Public(hex!("0000000000000000000000000000000000000000000000000000000000000005"));
}

// ----------------------------------------------------------------------------------------------------
//                                             Runtime
// ----------------------------------------------------------------------------------------------------
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<MockRuntime>;
type Block = frame_system::mocking::MockBlock<MockRuntime>;

frame_support::construct_runtime!(
	pub enum MockRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		LpTokenFactory: pallet_currency_factory::{Pallet, Storage, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage},
		// GovernanceRegistry: governance::{Pallet, Call, Storage, Event<T>},

		Vault: pallet_vault::{Pallet, Call, Storage, Event<T>},
		TokenizedOptions: pallet_tokenized_options::{Pallet, Call, Storage, Event<T>},
	}
);

// ----------------------------------------------------------------------------------------------------
//		Frame System Config
// ----------------------------------------------------------------------------------------------------

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for MockRuntime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

// ----------------------------------------------------------------------------------------------------
//		Composable Config
// ----------------------------------------------------------------------------------------------------
impl DeFiComposableConfig for MockRuntime {
	type Balance = Balance;
	type MayBeAssetId = AssetId;
}

// ----------------------------------------------------------------------------------------------------
//		Balances
// ----------------------------------------------------------------------------------------------------

parameter_types! {
	pub const BalanceExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for MockRuntime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = BalanceExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

// ----------------------------------------------------------------------------------------------------
//		Currency Factory
// ----------------------------------------------------------------------------------------------------

impl pallet_currency_factory::Config for MockRuntime {
	type Event = Event;
	type AssetId = CurrencyId;
	type AddOrigin = EnsureRoot<AccountId>;
	type ReserveOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

// ----------------------------------------------------------------------------------------------------
//		Tokens
// ----------------------------------------------------------------------------------------------------

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		0u128 as Balance
	};
}

impl orml_tokens::Config for MockRuntime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = ();
	type DustRemovalWhitelist = Everything;
}

// ----------------------------------------------------------------------------------------------------
//		Governance Registry
// ----------------------------------------------------------------------------------------------------

pub struct GovernanceRegistry;
impl composable_traits::governance::GovernanceRegistry<CurrencyId, AccountId>
	for GovernanceRegistry
{
	fn set(_k: CurrencyId, _value: composable_traits::governance::SignedRawOrigin<AccountId>) {}
}

impl
	GetByKey<
		CurrencyId,
		Result<SignedRawOrigin<sp_core::sr25519::Public>, sp_runtime::DispatchError>,
	> for GovernanceRegistry
{
	fn get(
		_k: &CurrencyId,
	) -> Result<SignedRawOrigin<sp_core::sr25519::Public>, sp_runtime::DispatchError> {
		Ok(SignedRawOrigin::Root)
	}
}

// ----------------------------------------------------------------------------------------------------
//		Assets
// ----------------------------------------------------------------------------------------------------

parameter_types! {
	pub const NativeAssetId: CurrencyId = PICA::ID;
}

ord_parameter_types! {
	pub const RootAccount: AccountId = accounts::ADMIN;
}

impl pallet_assets::Config for MockRuntime {
	type NativeAssetId = NativeAssetId;
	type GenerateCurrencyId = LpTokenFactory;
	type AssetId = CurrencyId;
	type Balance = Balance;
	type NativeCurrency = Balances;
	type MultiCurrency = Tokens;
	type WeightInfo = ();
	type AdminOrigin = EnsureSignedBy<RootAccount, AccountId>;
	type GovernanceRegistry = GovernanceRegistry;
}

// ----------------------------------------------------------------------------------------------------
//		Vault
// ----------------------------------------------------------------------------------------------------

parameter_types! {
	pub const MaxStrategies: usize = 255;
	pub const CreationDeposit: Balance = 10;
	pub const ExistentialDeposit: Balance = 1000;
	pub const RentPerBlock: Balance = 1;
	pub const MinimumDeposit: Balance = 0;
	pub const MinimumWithdrawal: Balance = 0;
	pub const VaultPalletId: PalletId = PalletId(*b"cubic___");
	pub const TombstoneDuration: u64 = 42;
}

impl pallet_vault::Config for MockRuntime {
	type Event = Event;
	type Balance = Balance;
	type MaxStrategies = MaxStrategies;
	type AssetId = CurrencyId;
	type CurrencyFactory = LpTokenFactory;
	type Convert = ConvertInto;
	type MinimumDeposit = MinimumDeposit;
	type MinimumWithdrawal = MinimumWithdrawal;
	type PalletId = VaultPalletId;
	type CreationDeposit = CreationDeposit;
	type ExistentialDeposit = ExistentialDeposit;
	type RentPerBlock = RentPerBlock;
	type NativeCurrency = Balances;
	type Currency = Tokens;
	type VaultId = VaultId;
	type TombstoneDuration = TombstoneDuration;
	type WeightInfo = ();
}

// ----------------------------------------------------------------------------------------------------
//		Options
// ----------------------------------------------------------------------------------------------------

parameter_types! {
	pub const TokenizedOptionsPalletId: PalletId = PalletId(*b"options_");
}

impl pallet_tokenized_options::Config for MockRuntime {
	type Event = Event;
	type WeightInfo = ();
	type CurrencyFactory = LpTokenFactory;
	type NativeCurrency = Balances;
	type MultiCurrency = Assets;
	type VaultId = VaultId;
	type Vault = Vault;
	type PalletId = TokenizedOptionsPalletId;
}

// ----------------------------------------------------------------------------------------------------
//		ExtBuilder
// ----------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct ExtBuilder {
	// balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage =
			frame_system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();

		let mut ext: sp_io::TestExternalities = storage.into();

		ext.execute_with(|| System::set_block_number(1));

		ext
	}
}
