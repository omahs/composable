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

// 	pub type AccountId = u128;

// 	pub static ADMIN: AccountId = 0;
// 	pub static ALICE: AccountId = 1;
// 	pub static BOB: AccountId = 2;
// 	pub static CHARLIE: AccountId = 3;
// 	pub static DAVE: AccountId = 4;
// 	pub static EVEN: AccountId = 5;
