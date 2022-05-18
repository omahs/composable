use crate as pallet_tokenized_options;
use crate::mock::currency::{defs::*, CurrencyId};
use accounts::*;
use composable_traits::{defi::DeFiComposableConfig, governance::SignedRawOrigin, oracle::Price};
use frame_support::traits::GenesisBuild;
use frame_support::{ord_parameter_types, parameter_types, traits::Everything, PalletId};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::{parameter_type_with_key, GetByKey};
use sp_core::{sr25519::Signature, H256};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{ConvertInto, Extrinsic as ExtrinsicT, IdentityLookup, Verify},
};

pub type BlockNumber = u64;
pub type AssetId = u128;
pub type Balance = u128;
pub type VaultId = u64;
pub type Amount = i128;
pub type Moment = u64;

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

// #[allow(dead_code)]
// pub mod accounts {
// 	pub type AccountId = u128;

// 	pub static ADMIN: AccountId = 0;
// 	pub static ALICE: AccountId = 1;
// 	pub static BOB: AccountId = 2;
// 	pub static CHARLIE: AccountId = 3;
// 	pub static DAVE: AccountId = 4;
// 	pub static EVEN: AccountId = 5;
// }

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
		Timestamp: pallet_timestamp,
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Oracle: pallet_oracle::{Pallet, Storage, Event<T>, Call},
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

impl pallet_timestamp::Config for MockRuntime {
	type Moment = Moment;
	type OnTimestampSet = ();
	// One second.
	type MinimumPeriod = frame_support::traits::ConstU64<1000>;
	type WeightInfo = ();
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
//		Oracle
// ----------------------------------------------------------------------------------------------------

parameter_types! {
	pub const StakeLock: u64 = 1;
	pub const MinStake: u64 = 1;
	pub const StalePrice: u64 = 2;
	pub const MaxAnswerBound: u32 = 5;
	pub const MaxAssetsCount: u32 = 2;
	pub const MaxHistory: u32 = 3;
	pub const MaxPrePrices: u32 = 12;
}

pub type Extrinsic = TestXt<Call, ()>;

impl frame_system::offchain::SigningTypes for MockRuntime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for MockRuntime
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for MockRuntime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

impl pallet_oracle::Config for MockRuntime {
	type Event = Event;
	type AuthorityId = pallet_oracle::crypto::BathurstStId;
	type Currency = Balances;
	type AssetId = AssetId;
	type PriceValue = Balance;
	type StakeLock = StakeLock;
	type StalePrice = StalePrice;
	type MinStake = MinStake;
	type AddOracle = EnsureSignedBy<RootAccount, sp_core::sr25519::Public>;
	type MaxAnswerBound = MaxAnswerBound;
	type MaxAssetsCount = MaxAssetsCount;
	type MaxHistory = MaxHistory;
	type MaxPrePrices = MaxPrePrices;
	type WeightInfo = ();
	type LocalAssets = ();
}

pub fn set_oracle_price(asset_id: AssetId, balance: Balance) {
	let price = Price { price: balance, block: System::block_number() };
	pallet_oracle::Prices::<MockRuntime>::insert(asset_id, price);
}

pub fn get_oracle_price(asset_id: AssetId, amount: Balance) -> Balance {
	<Oracle as composable_traits::oracle::Oracle>::get_price(asset_id, amount)
		.expect("Error retrieving price")
		.price
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

impl GetByKey<CurrencyId, Result<SignedRawOrigin<AccountId>, sp_runtime::DispatchError>>
	for GovernanceRegistry
{
	fn get(_k: &CurrencyId) -> Result<SignedRawOrigin<AccountId>, sp_runtime::DispatchError> {
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
	pub const MaxOptionNumber: u32 = 1000;
}

impl pallet_tokenized_options::Config for MockRuntime {
	type Event = Event;
	type PalletId = TokenizedOptionsPalletId;
	type WeightInfo = ();
	type Oracle = Oracle;
	type Moment = Moment;
	type Time = Timestamp;
	type MaxOptionNumber = MaxOptionNumber;
	type CurrencyFactory = LpTokenFactory;
	type NativeCurrency = Balances;
	type MultiCurrency = Assets;
	type VaultId = VaultId;
	type Vault = Vault;
}

// ----------------------------------------------------------------------------------------------------
//		ExtBuilder
// ----------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct ExtBuilder {
	pub native_balances: Vec<(AccountId, Balance)>,
	pub balances: Vec<(AccountId, AssetId, Balance)>,
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage =
			frame_system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();

		pallet_balances::GenesisConfig::<MockRuntime> { balances: self.native_balances }
			.assimilate_storage(&mut storage)
			.unwrap();

		orml_tokens::GenesisConfig::<MockRuntime> { balances: self.balances }
			.assimilate_storage(&mut storage)
			.unwrap();

		let mut ext: sp_io::TestExternalities = storage.into();

		ext.execute_with(|| System::set_block_number(1));

		ext
	}

	pub fn initialize_balances(
		mut self,
		balances: Vec<(AccountId, AssetId, Balance)>,
	) -> ExtBuilder {
		balances.into_iter().for_each(|(account, asset, balance)| {
			if asset == PICA::ID {
				self.native_balances.push((account, balance));
			} else {
				self.balances.push((account, asset, balance));
			}
		});

		self
	}
}
