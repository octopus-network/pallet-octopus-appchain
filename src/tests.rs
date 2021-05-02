use crate::mock::*;
use crate::*;
use sp_core::{
	offchain::{testing, OffchainExt, TransactionPoolExt},
};
use sp_keystore::{
	testing::KeyStore,
	{KeystoreExt, SyncCryptoStore},
};
use sp_runtime::RuntimeAppPublic;
 use std::sync::Arc;

fn expected_val_set() -> Option<ValidatorSet<AccountId>> {
	let id = hex::decode("482767e6ddb8f641c561a8e2ab8290cb7223b0637e25d076940250696c770350")
		.map(|b| AccountId::decode(&mut &b[..]))
		.unwrap()
		.unwrap();

	let alice = Validator {
		id: id,
		weight: 100,
	};

	let id = hex::decode("3cd8419984da9441b930ccf920d73cd853752a8d0db2e45e4eeeb4d84e090840")
		.map(|b| AccountId::decode(&mut &b[..]))
		.unwrap()
		.unwrap();

	let bob = Validator {
		id: id,
		weight: 100,
	};

	let id = hex::decode("5c15a0440e35941a1615168fd99eef5a4cf68d25febcb750b2a632ea70f1044b")
		.map(|b| AccountId::decode(&mut &b[..]))
		.unwrap()
		.unwrap();

	let charlie = Validator {
		id: id,
		weight: 100,
	};

	let id = hex::decode("f6fa4566989c54659c048c594f3770ffb18e18daf6529ec756a34dcc3d661d24")
		.map(|b| AccountId::decode(&mut &b[..]))
		.unwrap()
		.unwrap();

	let dave = Validator {
		id: id,
		weight: 100,
	};
	let expected_val_set = ValidatorSet {
		sequence_number: 0,
		validators: vec![alice, bob, charlie, dave],
	};
	Some(expected_val_set)
}

#[test]
fn should_make_http_call_and_parse_result() {
	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainExt::new(offchain));

	validator_set_response(&mut state.write());

	t.execute_with(|| {
		// when
		let val_set = OctopusAppchain::fetch_validator_set(b"octopus.testnet".to_vec(), 0, 1).ok();
		// then
		assert_eq!(val_set, expected_val_set());
	});
}

fn validator_set_response(state: &mut testing::OffchainState) {
	state.expect_request(testing::PendingRequest {
		method: "POST".into(),
		uri: "https://rpc.testnet.near.org".into(),
		headers: vec![("Content-Type".into(), "application/json".into())],
		body: br#"
		{
			"jsonrpc": "2.0",
			"id": "dontcare",
			"method": "query",
			"params": {
				"request_type": "call_function",
				"finality": "final",
				"account_id": "octopus.testnet",
				"method_name": "get_validator_set",
				"args_base64": "eyJhcHBjaGFpbl9pZCI6MCwic2VxX251bSI6MX0="
			}
		}"#.to_vec(),
		response: Some(br#"
			{
				"jsonrpc": "2.0",
				"result": {
					"result": [
						123,34,115,101,113,95,110,117,109,34,58,48,44,34,118,97,108,105,100,97,116,111,114,115,34,58,91,123,34,105,100,34,58,34,48,120,52,56,50,55,54,55,101,54,100,100,98,56,102,54,52,49,99,53,54,49,97,56,101,50,97,98,56,50,57,48,99,98,55,50,50,51,98,48,54,51,55,101,50,53,100,48,55,54,57,52,48,50,53,48,54,57,54,99,55,55,48,51,53,48,34,44,34,97,99,99,111,117,110,116,95,105,100,34,58,34,97,108,105,99,101,45,111,99,116,111,112,117,115,46,116,101,115,116,110,101,116,34,44,34,119,101,105,103,104,116,34,58,49,48,48,44,34,98,108,111,99,107,95,104,101,105,103,104,116,34,58,52,53,56,49,55,52,49,50,44,34,100,101,108,101,103,97,116,105,111,110,115,34,58,91,93,125,44,123,34,105,100,34,58,34,48,120,51,99,100,56,52,49,57,57,56,52,100,97,57,52,52,49,98,57,51,48,99,99,102,57,50,48,100,55,51,99,100,56,53,51,55,53,50,97,56,100,48,100,98,50,101,52,53,101,52,101,101,101,98,52,100,56,52,101,48,57,48,56,52,48,34,44,34,97,99,99,111,117,110,116,95,105,100,34,58,34,98,111,98,45,111,99,116,111,112,117,115,46,116,101,115,116,110,101,116,34,44,34,119,101,105,103,104,116,34,58,49,48,48,44,34,98,108,111,99,107,95,104,101,105,103,104,116,34,58,52,53,56,49,55,53,52,50,44,34,100,101,108,101,103,97,116,105,111,110,115,34,58,91,93,125,44,123,34,105,100,34,58,34,48,120,53,99,49,53,97,48,52,52,48,101,51,53,57,52,49,97,49,54,49,53,49,54,56,102,100,57,57,101,101,102,53,97,52,99,102,54,56,100,50,53,102,101,98,99,98,55,53,48,98,50,97,54,51,50,101,97,55,48,102,49,48,52,52,98,34,44,34,97,99,99,111,117,110,116,95,105,100,34,58,34,99,104,97,114,108,105,101,45,111,99,116,111,112,117,115,46,116,101,115,116,110,101,116,34,44,34,119,101,105,103,104,116,34,58,49,48,48,44,34,98,108,111,99,107,95,104,101,105,103,104,116,34,58,52,53,56,49,55,55,57,48,44,34,100,101,108,101,103,97,116,105,111,110,115,34,58,91,93,125,44,123,34,105,100,34,58,34,48,120,102,54,102,97,52,53,54,54,57,56,57,99,53,52,54,53,57,99,48,52,56,99,53,57,52,102,51,55,55,48,102,102,98,49,56,101,49,56,100,97,102,54,53,50,57,101,99,55,53,54,97,51,52,100,99,99,51,100,54,54,49,100,50,52,34,44,34,97,99,99,111,117,110,116,95,105,100,34,58,34,100,97,118,101,45,111,99,116,111,112,117,115,46,116,101,115,116,110,101,116,34,44,34,119,101,105,103,104,116,34,58,49,48,48,44,34,98,108,111,99,107,95,104,101,105,103,104,116,34,58,52,53,56,49,56,48,55,48,44,34,100,101,108,101,103,97,116,105,111,110,115,34,58,91,93,125,93,125
					],
					"logs": [],
					"block_height": 39225942,
					"block_hash": "BEZdFjq3G9x5TC6J6NYsfKFTTGBP6Hb5i8MCCKtBFXoA"
				},
				"id": "dontcare"
			}
			"#.to_vec()),
		sent: true,
		..Default::default()
	});
}

#[test]
fn parse_validator_set_works() {
	let test_data = vec![(
		r#"
			{
				"jsonrpc": "2.0",
				"result": {
					"result": [
						123,34,115,101,113,95,110,117,109,34,58,48,44,34,118,97,108,105,100,97,116,111,114,115,34,58,91,123,34,105,100,34,58,34,48,120,52,56,50,55,54,55,101,54,100,100,98,56,102,54,52,49,99,53,54,49,97,56,101,50,97,98,56,50,57,48,99,98,55,50,50,51,98,48,54,51,55,101,50,53,100,48,55,54,57,52,48,50,53,48,54,57,54,99,55,55,48,51,53,48,34,44,34,97,99,99,111,117,110,116,95,105,100,34,58,34,97,108,105,99,101,45,111,99,116,111,112,117,115,46,116,101,115,116,110,101,116,34,44,34,119,101,105,103,104,116,34,58,49,48,48,44,34,98,108,111,99,107,95,104,101,105,103,104,116,34,58,52,53,56,49,55,52,49,50,44,34,100,101,108,101,103,97,116,105,111,110,115,34,58,91,93,125,44,123,34,105,100,34,58,34,48,120,51,99,100,56,52,49,57,57,56,52,100,97,57,52,52,49,98,57,51,48,99,99,102,57,50,48,100,55,51,99,100,56,53,51,55,53,50,97,56,100,48,100,98,50,101,52,53,101,52,101,101,101,98,52,100,56,52,101,48,57,48,56,52,48,34,44,34,97,99,99,111,117,110,116,95,105,100,34,58,34,98,111,98,45,111,99,116,111,112,117,115,46,116,101,115,116,110,101,116,34,44,34,119,101,105,103,104,116,34,58,49,48,48,44,34,98,108,111,99,107,95,104,101,105,103,104,116,34,58,52,53,56,49,55,53,52,50,44,34,100,101,108,101,103,97,116,105,111,110,115,34,58,91,93,125,44,123,34,105,100,34,58,34,48,120,53,99,49,53,97,48,52,52,48,101,51,53,57,52,49,97,49,54,49,53,49,54,56,102,100,57,57,101,101,102,53,97,52,99,102,54,56,100,50,53,102,101,98,99,98,55,53,48,98,50,97,54,51,50,101,97,55,48,102,49,48,52,52,98,34,44,34,97,99,99,111,117,110,116,95,105,100,34,58,34,99,104,97,114,108,105,101,45,111,99,116,111,112,117,115,46,116,101,115,116,110,101,116,34,44,34,119,101,105,103,104,116,34,58,49,48,48,44,34,98,108,111,99,107,95,104,101,105,103,104,116,34,58,52,53,56,49,55,55,57,48,44,34,100,101,108,101,103,97,116,105,111,110,115,34,58,91,93,125,44,123,34,105,100,34,58,34,48,120,102,54,102,97,52,53,54,54,57,56,57,99,53,52,54,53,57,99,48,52,56,99,53,57,52,102,51,55,55,48,102,102,98,49,56,101,49,56,100,97,102,54,53,50,57,101,99,55,53,54,97,51,52,100,99,99,51,100,54,54,49,100,50,52,34,44,34,97,99,99,111,117,110,116,95,105,100,34,58,34,100,97,118,101,45,111,99,116,111,112,117,115,46,116,101,115,116,110,101,116,34,44,34,119,101,105,103,104,116,34,58,49,48,48,44,34,98,108,111,99,107,95,104,101,105,103,104,116,34,58,52,53,56,49,56,48,55,48,44,34,100,101,108,101,103,97,116,105,111,110,115,34,58,91,93,125,93,125
					],
					"logs": [],
					"block_height": 39225942,
					"block_hash": "BEZdFjq3G9x5TC6J6NYsfKFTTGBP6Hb5i8MCCKtBFXoA"
				},
				"id": "dontcare"
			}
			"#,
		expected_val_set(),
	)];

	for (json, expected) in test_data {
		assert_eq!(expected, OctopusAppchain::parse_validator_set(json));
	}
}

#[test]
fn extract_result_works() {
	let test_data = vec![(
		r#"
			{
				"jsonrpc": "2.0",
				"result": {
					"result": [
						111, 99, 116, 111, 112, 117, 115
					],
					"logs": [],
					"block_height": 39225942,
					"block_hash": "BEZdFjq3G9x5TC6J6NYsfKFTTGBP6Hb5i8MCCKtBFXoA"
				},
				"id": "dontcare"
			}
			"#,
		Some(vec![111, 99, 116, 111, 112, 117, 115]),
	)];

	for (json, expected) in test_data {
		assert_eq!(expected, OctopusAppchain::extract_result(json));
	}
}

#[test]
fn encode_args_works() {
	let test_data = vec![
		(
			0u32,
			0u32,
			Some(vec![
				101, 121, 74, 104, 99, 72, 66, 106, 97, 71, 70, 112, 98, 108, 57, 112, 90, 67, 73,
				54, 77, 67, 119, 105, 99, 50, 86, 120, 88, 50, 53, 49, 98, 83, 73, 54, 77, 72, 48,
				61,
			]),
		), // eyJhcHBjaGFpbl9pZCI6MCwic2VxX251bSI6MH0=
		(
			4294967295u32,
			4294967295u32,
			Some(vec![
				101, 121, 74, 104, 99, 72, 66, 106, 97, 71, 70, 112, 98, 108, 57, 112, 90, 67, 73,
				54, 78, 68, 73, 53, 78, 68, 107, 50, 78, 122, 73, 53, 78, 83, 119, 105, 99, 50, 86,
				120, 88, 50, 53, 49, 98, 83, 73, 54, 78, 68, 73, 53, 78, 68, 107, 50, 78, 122, 73,
				53, 78, 88, 48, 61,
			]),
		), // eyJhcHBjaGFpbl9pZCI6NDI5NDk2NzI5NSwic2VxX251bSI6NDI5NDk2NzI5NX0=
	];

	for (appchain_id, seq_num, expected) in test_data {
		assert_eq!(expected, OctopusAppchain::encode_args(appchain_id, seq_num));
	}
}

#[test]
fn should_submit_unsigned_transaction_on_chain() {
	const PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let keystore = KeyStore::new();

	SyncCryptoStore::sr25519_generate_new(
		&keystore,
		crate::crypto::Public::ID,
		Some(&format!("{}/hunter1", PHRASE)),
	)
	.unwrap();

	let public_key = SyncCryptoStore::sr25519_public_keys(&keystore, crate::crypto::Public::ID)
		.get(0)
		.unwrap()
		.clone();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	validator_set_response(&mut offchain_state.write());

	let payload = ValidatorSetPayload {
		public: <Test as SigningTypes>::Public::from(public_key),
		block_number: 1,
		val_set: expected_val_set().unwrap(),
	};

	// let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
	t.execute_with(|| {
		// when
		OctopusAppchain::fetch_and_update_validator_set(1, 1).unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		if let mock::Call::OctopusAppchain(crate::Call::submit_validator_set(body, signature)) = tx.call {
			assert_eq!(body, payload);

			let signature_valid = <ValidatorSetPayload<
				<Test as SigningTypes>::Public,
				<Test as frame_system::Config>::BlockNumber,
				<Test as frame_system::Config>::AccountId,
			> as SignedPayload<Test>>::verify::<<Test as Config>::AppCrypto>(
				&payload, signature
			);

			assert!(signature_valid);
		}
	});
}