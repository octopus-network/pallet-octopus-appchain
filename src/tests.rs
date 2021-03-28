use crate as octopus_appchain;
use crate::*;
use codec::Decode;
use frame_support::parameter_types;
use sp_core::{
	offchain::{testing, OffchainExt, TransactionPoolExt},
	H256,
};
use sp_keystore::{
	testing::KeyStore,
	{KeystoreExt, SyncCryptoStore},
};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature, RuntimeAppPublic,
};
use std::sync::Arc;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// For testing the module, we construct a mock runtime.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		OctopusAppchain: octopus_appchain::{Module, Call, Storage, Event<T>, ValidateUnsigned},
	}
);

type Signature = MultiSignature;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
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
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

type Extrinsic = TestXt<Call, ()>;

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
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

pub struct OctopusAppCrypto;

impl frame_system::offchain::AppCrypto<<Signature as Verify>::Signer, Signature> for OctopusAppCrypto {
	type RuntimeAppPublic = crypto::AuthorityId;
	type GenericSignature = sp_core::sr25519::Signature;
	type GenericPublic = sp_core::sr25519::Public;
}

parameter_types! {
	pub const AppchainId: ChainId = 0;
	pub const Motherchain: MotherchainType = MotherchainType::NEAR;
	pub const GracePeriod: u64 = 5;
	pub const UnsignedPriority: u64 = 1 << 20;
}

impl Config for Test {
	type Event = Event;
	type AppCrypto = OctopusAppCrypto;
	type Call = Call;
	type AppchainId = AppchainId;
	type Motherchain = Motherchain;
	const RELAY_CONTRACT_NAME: &'static [u8] = b"octopus.testnet";
	type GracePeriod = GracePeriod;
	type UnsignedPriority = UnsignedPriority;
}

fn expected_val_set() -> Option<ValidatorSet<AccountId>> {
	let id = hex::decode("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
		.map(|b| AccountId::decode(&mut &b[..]))
		.unwrap()
		.unwrap();

	let ocw_id = hex::decode("306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20")
		.map(|b| AccountId::decode(&mut &b[..]))
		.unwrap()
		.unwrap();

	let alice = Validator {
		id: id,
		ocw_id: ocw_id,
		weight: 101,
	};

	let id = hex::decode("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22")
		.map(|b| AccountId::decode(&mut &b[..]))
		.unwrap()
		.unwrap();

	let ocw_id = hex::decode("1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c")
		.map(|b| AccountId::decode(&mut &b[..]))
		.unwrap()
		.unwrap();

	let charlie = Validator {
		id: id,
		ocw_id: ocw_id,
		weight: 102,
	};
	let expected_val_set = ValidatorSet {
		sequence_number: 1,
		validators: vec![charlie, alice],
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
						123, 34, 97, 112, 112, 99, 104, 97, 105, 110, 95, 105, 100, 34, 58, 48, 44, 34, 115, 101, 113, 117, 101, 110, 99, 101, 95, 110, 117, 109, 98, 101, 114, 34, 58, 49, 44, 34, 118, 97, 108, 105, 100, 97, 116, 111, 114, 115, 34, 58, 91, 123, 34, 97, 99, 99, 111, 117, 110, 116, 95, 105, 100, 34, 58, 34, 99, 104, 97, 114, 108, 105, 101, 45, 111, 99, 116, 111, 112, 117, 115, 46, 116, 101, 115, 116, 110, 101, 116, 34, 44, 34, 105, 100, 34, 58, 34, 48, 120, 57, 48, 98, 53, 97, 98, 50, 48, 53, 99, 54, 57, 55, 52, 99, 57, 101, 97, 56, 52, 49, 98, 101, 54, 56, 56, 56, 54, 52, 54, 51, 51, 100, 99, 57, 99, 97, 56, 97, 51, 53, 55, 56, 52, 51, 101, 101, 97, 99, 102, 50, 51, 49, 52, 54, 52, 57, 57, 54, 53, 102, 101, 50, 50, 34, 44, 34, 111, 99, 119, 95, 105, 100, 34, 58, 34, 48, 120, 49, 99, 98, 100, 50, 100, 52, 51, 53, 51, 48, 97, 52, 52, 55, 48, 53, 97, 100, 48, 56, 56, 97, 102, 51, 49, 51, 101, 49, 56, 102, 56, 48, 98, 53, 51, 101, 102, 49, 54, 98, 51, 54, 49, 55, 55, 99, 100, 52, 98, 55, 55, 98, 56, 52, 54, 102, 50, 97, 53, 102, 48, 55, 99, 34, 44, 34, 119, 101, 105, 103, 104, 116, 34, 58, 49, 48, 50, 44, 34, 115, 116, 97, 107, 101, 100, 95, 97, 109, 111, 117, 110, 116, 34, 58, 49, 48, 50, 44, 34, 98, 108, 111, 99, 107, 95, 104, 101, 105, 103, 104, 116, 34, 58, 52, 48, 56, 53, 55, 52, 52, 51, 44, 34, 100, 101, 108, 101, 103, 97, 116, 105, 111, 110, 115, 34, 58, 91, 93, 125, 44, 123, 34, 97, 99, 99, 111, 117, 110, 116, 95, 105, 100, 34, 58, 34, 97, 108, 105, 99, 101, 45, 111, 99, 116, 111, 112, 117, 115, 46, 116, 101, 115, 116, 110, 101, 116, 34, 44, 34, 105, 100, 34, 58, 34, 48, 120, 100, 52, 51, 53, 57, 51, 99, 55, 49, 53, 102, 100, 100, 51, 49, 99, 54, 49, 49, 52, 49, 97, 98, 100, 48, 52, 97, 57, 57, 102, 100, 54, 56, 50, 50, 99, 56, 53, 53, 56, 56, 53, 52, 99, 99, 100, 101, 51, 57, 97, 53, 54, 56, 52, 101, 55, 97, 53, 54, 100, 97, 50, 55, 100, 34, 44, 34, 111, 99, 119, 95, 105, 100, 34, 58, 34, 48, 120, 51, 48, 54, 55, 50, 49, 50, 49, 49, 100, 53, 52, 48, 52, 98, 100, 57, 100, 97, 56, 56, 101, 48, 50, 48, 52, 51, 54, 48, 97, 49, 97, 57, 97, 98, 56, 98, 56, 55, 99, 54, 54, 99, 49, 98, 99, 50, 102, 99, 100, 100, 51, 55, 102, 51, 99, 50, 50, 50, 50, 99, 99, 50, 48, 34, 44, 34, 119, 101, 105, 103, 104, 116, 34, 58, 49, 48, 49, 44, 34, 115, 116, 97, 107, 101, 100, 95, 97, 109, 111, 117, 110, 116, 34, 58, 49, 48, 49, 44, 34, 98, 108, 111, 99, 107, 95, 104, 101, 105, 103, 104, 116, 34, 58, 52, 48, 56, 53, 55, 49, 55, 57, 44, 34, 100, 101, 108, 101, 103, 97, 116, 105, 111, 110, 115, 34, 58, 91, 93, 125, 93, 125
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
						123, 34, 97, 112, 112, 99, 104, 97, 105, 110, 95, 105, 100, 34, 58, 48, 44, 34, 115, 101, 113, 117, 101, 110, 99, 101, 95, 110, 117, 109, 98, 101, 114, 34, 58, 49, 44, 34, 118, 97, 108, 105, 100, 97, 116, 111, 114, 115, 34, 58, 91, 123, 34, 97, 99, 99, 111, 117, 110, 116, 95, 105, 100, 34, 58, 34, 99, 104, 97, 114, 108, 105, 101, 45, 111, 99, 116, 111, 112, 117, 115, 46, 116, 101, 115, 116, 110, 101, 116, 34, 44, 34, 105, 100, 34, 58, 34, 48, 120, 57, 48, 98, 53, 97, 98, 50, 48, 53, 99, 54, 57, 55, 52, 99, 57, 101, 97, 56, 52, 49, 98, 101, 54, 56, 56, 56, 54, 52, 54, 51, 51, 100, 99, 57, 99, 97, 56, 97, 51, 53, 55, 56, 52, 51, 101, 101, 97, 99, 102, 50, 51, 49, 52, 54, 52, 57, 57, 54, 53, 102, 101, 50, 50, 34, 44, 34, 111, 99, 119, 95, 105, 100, 34, 58, 34, 48, 120, 49, 99, 98, 100, 50, 100, 52, 51, 53, 51, 48, 97, 52, 52, 55, 48, 53, 97, 100, 48, 56, 56, 97, 102, 51, 49, 51, 101, 49, 56, 102, 56, 48, 98, 53, 51, 101, 102, 49, 54, 98, 51, 54, 49, 55, 55, 99, 100, 52, 98, 55, 55, 98, 56, 52, 54, 102, 50, 97, 53, 102, 48, 55, 99, 34, 44, 34, 119, 101, 105, 103, 104, 116, 34, 58, 49, 48, 50, 44, 34, 115, 116, 97, 107, 101, 100, 95, 97, 109, 111, 117, 110, 116, 34, 58, 49, 48, 50, 44, 34, 98, 108, 111, 99, 107, 95, 104, 101, 105, 103, 104, 116, 34, 58, 52, 48, 56, 53, 55, 52, 52, 51, 44, 34, 100, 101, 108, 101, 103, 97, 116, 105, 111, 110, 115, 34, 58, 91, 93, 125, 44, 123, 34, 97, 99, 99, 111, 117, 110, 116, 95, 105, 100, 34, 58, 34, 97, 108, 105, 99, 101, 45, 111, 99, 116, 111, 112, 117, 115, 46, 116, 101, 115, 116, 110, 101, 116, 34, 44, 34, 105, 100, 34, 58, 34, 48, 120, 100, 52, 51, 53, 57, 51, 99, 55, 49, 53, 102, 100, 100, 51, 49, 99, 54, 49, 49, 52, 49, 97, 98, 100, 48, 52, 97, 57, 57, 102, 100, 54, 56, 50, 50, 99, 56, 53, 53, 56, 56, 53, 52, 99, 99, 100, 101, 51, 57, 97, 53, 54, 56, 52, 101, 55, 97, 53, 54, 100, 97, 50, 55, 100, 34, 44, 34, 111, 99, 119, 95, 105, 100, 34, 58, 34, 48, 120, 51, 48, 54, 55, 50, 49, 50, 49, 49, 100, 53, 52, 48, 52, 98, 100, 57, 100, 97, 56, 56, 101, 48, 50, 48, 52, 51, 54, 48, 97, 49, 97, 57, 97, 98, 56, 98, 56, 55, 99, 54, 54, 99, 49, 98, 99, 50, 102, 99, 100, 100, 51, 55, 102, 51, 99, 50, 50, 50, 50, 99, 99, 50, 48, 34, 44, 34, 119, 101, 105, 103, 104, 116, 34, 58, 49, 48, 49, 44, 34, 115, 116, 97, 107, 101, 100, 95, 97, 109, 111, 117, 110, 116, 34, 58, 49, 48, 49, 44, 34, 98, 108, 111, 99, 107, 95, 104, 101, 105, 103, 104, 116, 34, 58, 52, 48, 56, 53, 55, 49, 55, 57, 44, 34, 100, 101, 108, 101, 103, 97, 116, 105, 111, 110, 115, 34, 58, 91, 93, 125, 93, 125
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
		if let Call::OctopusAppchain(crate::Call::submit_validator_set(body, signature)) = tx.call {
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