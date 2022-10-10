//! ## GRANDPA SYS crate
//!
//! The purpose of this crate is to create an interface to facilitate
//! FFI between Go and `ics10-grandpa` which is written in Rust.

#![allow(clippy::all)]

mod utils;

// use ics10_grandpa::client_def::GrandpaClient;
use ics10_grandpa::client_state::ClientState;

use sp_core::ed25519;
use sp_runtime::{app_crypto::RuntimePublic, traits::BlakeTwo256};

#[derive(Clone, Default, PartialEq, Debug, Eq)]
pub struct HostFunctionsManager;

impl grandpa_light_client_primitives::HostFunctions for HostFunctionsManager {
	fn ed25519_verify(sig: &ed25519::Signature, msg: &[u8], pub_key: &ed25519::Public) -> bool {
		pub_key.verify(&msg, sig)
	}
}

impl light_client_common::HostFunctions for HostFunctionsManager {
	type BlakeTwo256 = BlakeTwo256;
}

pub extern "C" fn create_client_state() -> *const ClientState<HostFunctionsManager> {
	let client_state = ClientState::default();
	let boxed_client_state = Box::new(client_state);
	Box::into_raw(boxed_client_state)
}

#[cfg(test)]
mod tests {
	use super::*;
	use ibc::core::ics02_client::height::Height;

	#[test]
	fn test_create_client_state_dummy() {
		let client_state = create_client_state();
		unsafe {
			assert_eq!(client_state.as_ref().unwrap().latest_height(), Height::default());
		}
	}
}
