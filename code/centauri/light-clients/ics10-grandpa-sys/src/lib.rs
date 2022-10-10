//! ## GRANDPA SYS crate
//!
//! The purpose of this crate is to create an interface to facilitate
//! FFI between Go and `ics10-grandpa` which is written in Rust.

#![allow(clippy::all)]

mod utils;

use std::ops::Deref;

use grandpa_light_client_primitives::justification::GrandpaJustification;
// use ics10_grandpa::client_def::GrandpaClient;
use ibc::core::ics02_client::height::Height;
use ics10_grandpa::{client_message::RelayChainHeader, client_state::ClientState};

use codec::Decode;

use sp_core::ed25519;
use sp_runtime::{app_crypto::RuntimePublic, traits::BlakeTwo256};

#[derive(Clone, Default, PartialEq, Debug, Eq)]
pub struct HostFunctionsManager;

// FFI-safe wrapper of Header
#[repr(C)]
#[derive(Debug)]
pub struct HeightWrapper(Height);

impl grandpa_light_client_primitives::HostFunctions for HostFunctionsManager {
	fn ed25519_verify(sig: &ed25519::Signature, msg: &[u8], pub_key: &ed25519::Public) -> bool {
		pub_key.verify(&msg, sig)
	}
}

impl light_client_common::HostFunctions for HostFunctionsManager {
	type BlakeTwo256 = BlakeTwo256;
}

/// Opaque pointer to make my life easier in C/Go world
#[repr(C)]
pub struct ClientStateWrapper {
	private: ClientState<HostFunctionsManager>,
}

impl Deref for ClientStateWrapper {
	type Target = ClientState<HostFunctionsManager>;
	fn deref(&self) -> &Self::Target {
		&self.private
	}
}

#[no_mangle]
pub extern "C" fn create_client_state() -> *const ClientStateWrapper {
	let client_state = ClientStateWrapper { private: ClientState::default() };
	let boxed_client_state = Box::new(client_state);
	Box::into_raw(boxed_client_state)
}

#[no_mangle]
pub extern "C" fn client_state_latest_height(client_state: *const libc::c_void) -> HeightWrapper {
	let client_state = client_state as *const ClientStateWrapper;
	unsafe { HeightWrapper(client_state.as_ref().unwrap().latest_height()) }
}

#[no_mangle]
pub extern "C" fn decode_justification_bytes(
	justification_bytes: *const libc::c_void,
) -> *const libc::c_void {
	let justification_bytes = justification_bytes as *const Vec<u8>;

	let mut justification_bytes = unsafe { justification_bytes.as_ref().unwrap() };

	let justification =
		GrandpaJustification::<RelayChainHeader>::decode(&mut &justification_bytes[..])
			.expect("Failed to decode justification");

	Box::into_raw(Box::new(justification)) as _
}

#[cfg(test)]
mod tests {
	use super::*;
	use grandpa_light_client_primitives::justification::GrandpaJustification;
	use ibc::core::ics02_client::height::Height;
	use ics10_grandpa::client_message::RelayChainHeader;

	pub type Justification = GrandpaJustification<RelayChainHeader>;

	#[test]
	fn test_create_client_state_dummy() {
		let client_state = create_client_state();
		unsafe {
			assert_eq!(client_state.as_ref().unwrap().latest_height(), Height::default());
		}
	}

	#[test]
	fn test_client_state_latest_height() {
		// simple test to be able to call a method from a struct that's "hidden" as an opaque raw
		// pointer
		let client_state = create_client_state();
		let client_state_ptr = &client_state as *const _;
		unsafe {
			assert_eq!(
				client_state_latest_height(client_state_ptr as *const libc::c_void).0,
				Height::default()
			);
		}
	}

	#[test]
	#[should_panic]
	fn test_decode_justufication_bytes_fails() {
		let justification_bytes = Box::into_raw(Box::new(vec![1, 2, 3])) as _;
		let _ = decode_justification_bytes(justification_bytes);
	}
}
