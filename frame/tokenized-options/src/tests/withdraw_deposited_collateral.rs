use crate::mock::runtime::{
	Assets, Balance, Balances, Event, ExtBuilder, MockRuntime, Moment, Origin, System,
	TokenizedOptions, Vault,
};

use crate::mock::accounts::*;
use crate::mock::assets::*;

use crate::pallet::{self, OptionHashToOptionId, Sellers};
use crate::tests::*;

use composable_traits::tokenized_options::TokenizedOptions as TokenizedOptionsTrait;
use composable_traits::vault::Vault as VaultTrait;

use frame_support::assert_noop;
use frame_support::traits::fungibles::Inspect;
use frame_system::ensure_signed;
use sp_core::{sr25519::Public, H256};

// ----------------------------------------------------------------------------------------------------
//		Withdraw Deposited Collateral Tests
// ----------------------------------------------------------------------------------------------------
