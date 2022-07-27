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

pub const fn account_id_from_u64(n: u64) -> AccountId {
	let bytes_src = n.to_be_bytes();
	let mut bytes_dst = [0u8; 32];
	let mut k = 0;
	while k < bytes_src.len() {
		bytes_dst[k + 24] = bytes_src[k];
		k += 1;
	}
	Public(bytes_dst)
}
