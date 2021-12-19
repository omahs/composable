use frame_support::{traits::Everything, parameter_types};
use sp_core::H256;
use sp_runtime::traits::{IdentityLookup, BlakeTwo256};


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

frame_support::construct_runtime!{
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System : frame_system::{Pallet, Call, Config, Storage, Event<T>},
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