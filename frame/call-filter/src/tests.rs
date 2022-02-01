#![cfg(test)]

use super::*;
use beefy_primitives::Commitment;
use mock::{Event, *};
use sp_core::{Pair, H256};
use sp_runtime::traits::BadOrigin;
use support::{assert_noop, assert_ok};

const BALANCE_TRANSFER: &<Runtime as system::Config>::Call =
	&mock::Call::Balances(pallet_balances::Call::transfer { dest: ALICE, value: 10 });
#[test]
fn pause_transaction_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		let balances_transfer = CallFilterEntry {
			pallet_name: b"Balances".to_vec(),
			function_name: b"transfer".to_vec(),
		};
		assert_noop!(Filter::disable(Origin::signed(5), balances_transfer.clone()), BadOrigin);

		assert_eq!(Filter::disabled_calls(&balances_transfer), None);
		assert_ok!(Filter::disable(Origin::signed(1), balances_transfer.clone()));
		System::assert_last_event(Event::Filter(crate::Event::Disabled {
			entry: balances_transfer.clone(),
		}));
		assert_eq!(Filter::disabled_calls(&balances_transfer), Some(()));

		let filter_pause =
			CallFilterEntry { pallet_name: b"Filter".to_vec(), function_name: b"disable".to_vec() };
		let filter_pause_2 = CallFilterEntry {
			pallet_name: b"Filter".to_vec(),
			function_name: b"another_call".to_vec(),
		};

		assert_noop!(
			Filter::disable(Origin::signed(1), filter_pause),
			Error::<Runtime>::CannotDisable
		);
		assert_noop!(
			Filter::disable(Origin::signed(1), filter_pause_2),
			Error::<Runtime>::CannotDisable
		);

		let other = CallFilterEntry {
			pallet_name: b"OtherPallet".to_vec(),
			function_name: b"disable".to_vec(),
		};
		assert_ok!(Filter::disable(Origin::signed(1), other));
	});
}

#[test]
fn enable_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		let balances_transfer = CallFilterEntry {
			pallet_name: b"Balances".to_vec(),
			function_name: b"transfer".to_vec(),
		};

		assert_ok!(Filter::disable(Origin::signed(1), balances_transfer.clone()));
		assert_eq!(Filter::disabled_calls(&balances_transfer), Some(()));

		assert_noop!(Filter::enable(Origin::signed(5), balances_transfer.clone()), BadOrigin);

		assert_ok!(Filter::enable(Origin::signed(1), balances_transfer.clone()));
		System::assert_last_event(Event::Filter(crate::Event::Enabled {
			entry: balances_transfer.clone(),
		}));
		assert_eq!(Filter::disabled_calls(&balances_transfer), None);
	});
}

#[test]
fn paused_transaction_filter_work() {
	ExtBuilder::default().build().execute_with(|| {
		let balances_transfer = CallFilterEntry {
			pallet_name: b"Balances".to_vec(),
			function_name: b"transfer".to_vec(),
		};

		assert!(!Filter::contains(BALANCE_TRANSFER));
		assert_ok!(Filter::disable(Origin::signed(1), balances_transfer.clone()));

		assert!(Filter::contains(BALANCE_TRANSFER));
		assert_ok!(Filter::enable(Origin::signed(1), balances_transfer));

		assert!(!Filter::contains(BALANCE_TRANSFER));
	});
}

#[test]
fn test_crypto_signature_recovery() {
	use codec::Encode;
	let commitment = Commitment::<u32, H256> {
		payload: H256::from_slice(
			&hex::decode("b44ddc7af2d75203036f2ab747701de9d54b9b31461df4c8afcc63d12282c733")
				.unwrap(),
		),
		block_number: 785,
		validator_set_id: 0,
	};
	let hash = sp_core::keccak_256(&commitment.encode());
	let signatures = [
		"c15f45a0c5246a92fd797cf45f716e7d12aad3919b6bae7ce76f9f78851048c516a9cbd8d2decf12dcb152c0a30ac603d09f80a57e58273fc19d357427e1925f01",
		"e477ea675dd5428cbddc51b4d2aa070d79504da5923d4c2149b9fc182c1558ce0b63299eed8f1695f9b3bfd0adc2a101dbcc49b85a7cd18c093087e09e8f3d2700",
		"984b00f53766e4ba63cb48462f141aaa07cde00196d8f564a0d9d29e9324f4e8731c3ff0288359d07909d800a3571920b94f64dfba1dea3932d0a8888c9775dc00",
		"1eadc75000162919a9cfff0d2f6b1b8892fa5c3abdeba0183224a8045e1aa49660766934b0a54d81358f439217929e3ff5d66f7a84f6f69f0b04fa6947fdc74901",
	];

	for sig in signatures {
		let sig = sp_core::ecdsa::Signature::from_slice(&hex::decode(sig).unwrap());
		let public = sig.recover_prehashed(&hash).unwrap();
		let decompress = libsecp256k1::PublicKey::parse_slice(
			public.as_ref(),
			Some(libsecp256k1::PublicKeyFormat::Compressed),
		)
		// uncompress the key
		.map(|pub_key| pub_key.serialize().to_vec())
		// now convert to ETH address
		.map(|uncompressed| sp_core::keccak_256(&uncompressed[1..])[12..].to_vec())
		.unwrap_or_default();
		println!("\nrecovered decompress: {}", hex::encode(decompress));
	}
	let keys = vec!["//Alice", "//Bob", "//Charlie", "//Dave", "//Ferdie"];
	for key in keys {
		let (key, seed) = sp_core::ecdsa::Pair::from_string_with_seed(key, None).unwrap();
		let decompress = libsecp256k1::PublicKey::parse_slice(
			key.public().as_ref(),
			Some(libsecp256k1::PublicKeyFormat::Compressed),
		)
		// uncompress the key
		.map(|pub_key| pub_key.serialize().to_vec())
		// now convert to ETH address
		.map(|uncompressed| sp_core::keccak_256(&uncompressed[1..])[12..].to_vec())
		.unwrap_or_default();
		println!(
			"\naddress: 0x{}, \npubkey: 0x{}, \nseed: 0x{}\ndecompress: {}",
			hex::encode(&sp_core::keccak_256(&key.public().as_ref()[1..])[12..]),
			hex::encode(key.public().as_ref()),
			hex::encode(seed.unwrap().as_ref()),
			hex::encode(decompress),
		);
		let signature = key.sign("Hello world".as_bytes());
		let public = signature.recover("Hello world".as_bytes()).unwrap();
		println!(
			"\nRecovered: 0x{}\n",
			hex::encode(&sp_core::keccak_256(&public.as_ref()[1..])[12..])
		);
	}
}
