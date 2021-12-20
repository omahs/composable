use composable_traits::governance::GovernanceRegistry;
use frame_support::{traits::{Everything, }, parameter_types};
use orml_traits::parameter_type_with_key;
use sp_core::{H256, ed25519::Signature};
use sp_runtime::{traits::{IdentityLookup, BlakeTwo256, Verify, IdentifyAccount}, testing::Header};
use crate::{self as pallet_dutch_auction, *};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;
type Balance = u64;

type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

frame_support::construct_runtime!{
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System : frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage},
        Tokens: orml_tokens::{Pallet, Call, Storage, Config<T>, Event<T>},
		
		LpTokenFactory: pallet_currency_factory::{Pallet, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage},

    }
}

parameter_types! {
    pub const SS58Prefix: u8 = 42;
    pub const BlockHashCount: u64 = 250;

}

impl frame_system::Config for Runtime {
    type BaseCallFilter = Everything;

    type BlockWeights = ();

    type BlockLength = ();

    type Origin = Origin;

    type Call = Call;

    type Index = u64;

    type BlockNumber = u64;

    type Hash = H256;

    type Hashing = BlakeTwo256;

    type AccountId = AccountId;

    type Lookup = IdentityLookup<Self::AccountId>;

    type Header = Header;

    type Event = Event;

    type BlockHashCount = BlockHashCount;

    type DbWeight = ();

    type Version = ();

    type PalletInfo = PalletInfo;

    type AccountData = pallet_balances::AccountData<Balance>;

    type OnNewAccount = ();

    type OnKilledAccount = ();

    type SystemWeightInfo = ();

    type SS58Prefix = SS58Prefix;

    type OnSetCode = ();
}


parameter_types! {
	pub const ExistentialDeposit: Balance = 1000;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}


pub const MILLISECS_PER_BLOCK: u64 = 6000;

parameter_types! {
	pub const MinimumPeriod: u64 = MILLISECS_PER_BLOCK / 2;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}


parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: MockCurrencyId| -> Balance {
		Zero::zero()
	};
}

impl orml_tokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = MockCurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = ();
	type DustRemovalWhitelist = Everything;
}

impl GovernanceRegistry<MockCurrencyId, AccountId> for () {
	fn set(_k: MockCurrencyId, _value: composable_traits::governance::SignedRawOrigin<AccountId>) {}
}

impl
	GetByKey<
		MockCurrencyId,
		Result<SignedRawOrigin<sp_core::sr25519::Public>, sp_runtime::DispatchError>,
	> for ()
{
	fn get(
		_k: &MockCurrencyId,
	) -> Result<SignedRawOrigin<sp_core::sr25519::Public>, sp_runtime::DispatchError> {
		Ok(SignedRawOrigin::Root)
	}
}


// parameter_type_with_key! {
// 	pub ExistentialDeposits: |_currency_id: MockCurrencyId| -> Balance {
// 		Zero::zero()
// 	};
// }


impl pallet_assets::Config for Test {
	type NativeAssetId = NativeAssetId;
	type GenerateCurrencyId = LpTokenFactory;
	type AssetId = MockCurrencyId;
	type Balance = Balance;
	type NativeCurrency = Balances;
	type MultiCurrency = Tokens;
	type WeightInfo = ();
	type AdminOrigin = EnsureSignedBy<RootAccount, AccountId>;
	type GovernanceRegistry = ();
}


parameter_types! {
	pub const DynamicCurrencyIdInitial: MockCurrencyId = MockCurrencyId::LpToken(0);
}

impl pallet_currency_factory::Config for Test {
	type Event = Event;
	type DynamicCurrencyId = MockCurrencyId;
	type DynamicCurrencyIdInitial = DynamicCurrencyIdInitial;
}


// impl pallet_dutch_auction::Config for Runtime {
//     type Event = Event;

//     type UnixTime = ;

//     type OrderId;

//     type MultiCurrency;

//     type WeightInfo;

//     type PalletId;

//     type NativeCurrency;
// }