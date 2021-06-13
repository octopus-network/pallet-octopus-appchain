use super::*;

#[derive(Serialize, Deserialize)]
struct Response {
	jsonrpc: String,
	result: ResponseResult,
	id: String,
}

#[derive(Serialize, Deserialize)]
struct ResponseResult {
	result: Vec<u8>,
	logs: Vec<u8>,
	block_height: u64,
	block_hash: String,
}

impl<T: Config> Pallet<T> {
	/// Fetch the validator set of a specified appchain with seq_num from relay contract.
	pub(super) fn fetch_validator_set(
		relay_contract: Vec<u8>,
		appchain_id: Vec<u8>,
		seq_num: u32,
	) -> Result<ValidatorSet<<T as frame_system::Config>::AccountId>, http::Error> {
		// We want to keep the offchain worker execution time reasonable, so we set a hard-coded
		// deadline to 2s to complete the external call.
		// You can also wait idefinitely for the response, however you may still get a timeout
		// coming from the host machine.
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
		// Initiate an external HTTP GET request.
		// This is using high-level wrappers from `sp_runtime`, for the low-level calls that
		// you can find in `sp_io`. The API is trying to be similar to `reqwest`, but
		// since we are running in a custom WASM execution environment we can't simply
		// import the library here.
		let args = Self::encode_args(appchain_id, seq_num).ok_or_else(|| {
			log::info!("🐙 Encode args error");
			http::Error::Unknown
		})?;

		let mut body = br#"
		{
			"jsonrpc": "2.0",
			"id": "dontcare",
			"method": "query",
			"params": {
				"request_type": "call_function",
				"finality": "final",
				"account_id": ""#
			.to_vec();
		body.extend(&relay_contract);
		body.extend(
			br#"",
				"method_name": "get_validator_set",
				"args_base64": ""#,
		);
		body.extend(&args);
		body.extend(
			br#""
			}
		}"#,
		);
		let request = http::Request::default()
			.method(http::Method::Post)
			.url("https://rpc.testnet.near.org")
			.body(vec![body])
			.add_header("Content-Type", "application/json");
		// We set the deadline for sending of the request, note that awaiting response can
		// have a separate deadline. Next we send the request, before that it's also possible
		// to alter request headers or stream body content in case of non-GET requests.
		let pending = request
			.deadline(deadline)
			.send()
			.map_err(|_| http::Error::IoError)?;

		// The request is already being processed by the host, we are free to do anything
		// else in the worker (we can send multiple concurrent requests too).
		// At some point however we probably want to check the response though,
		// so we can block current thread and wait for it to finish.
		// Note that since the request is being driven by the host, we don't have to wait
		// for the request to have it complete, we will just not read the response.
		let response = pending
			.try_wait(deadline)
			.map_err(|_| http::Error::DeadlineReached)??;
		// Let's check the status code before we proceed to reading the response.
		if response.code != 200 {
			log::info!("🐙 Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown);
		}

		// Next we want to fully read the response body and collect it to a vector of bytes.
		// Note that the return object allows you to read the body in chunks as well
		// with a way to control the deadline.
		let body = response.body().collect::<Vec<u8>>();
		log::info!("🐙 body: {:?}", body);

		// TODO
		let json_response: Response = serde_json::from_slice(&body).unwrap();

		let val_set = match Self::parse_validator_set(json_response.result.result.clone()) {
			Some(val_set) => Ok(val_set),
			None => {
				log::info!(
					"🐙 Unable to extract validator set from the response: {:?}",
					json_response.result.result
				);
				Err(http::Error::Unknown)
			}
		}?;

		log::info!("🐙 Got validator set: {:?}", val_set);

		Ok(val_set)
	}

	fn encode_args(appchain_id: Vec<u8>, seq_num: u32) -> Option<Vec<u8>> {
		let a = String::from("{\"appchain_id\":\"");
		let appchain_id = sp_std::str::from_utf8(&appchain_id).expect("octopus team will ensure that the appchain_id of a live appchain is a valid UTF8 string; qed");
		let b = String::from("\",\"seq_num\":");
		let seq_num = seq_num.to_string();
		let c = String::from("}");
		let json = a + &appchain_id + &b + &seq_num + &c;
		let res = base64::encode(json).into_bytes();
		Some(res)
	}

	fn parse_validator_set(
		result: Vec<u8>,
	) -> Option<ValidatorSet<<T as frame_system::Config>::AccountId>> {
		let result_str = sp_std::str::from_utf8(&result)
			.map_err(|_| {
				log::info!("🐙 No UTF8 result");
				Option::<ValidatorSet<<T as frame_system::Config>::AccountId>>::None
			})
			.ok()?;

		log::info!("🐙 Got result: {:?}", result_str);
		let mut val_set: ValidatorSet<<T as frame_system::Config>::AccountId> = ValidatorSet {
			sequence_number: 0,
			validators: vec![],
		};
		let val = lite_json::parse_json(result_str);
		val.ok().and_then(|v| match v {
			JsonValue::Object(obj) => {
				val_set.sequence_number = obj
					.clone()
					.into_iter()
					.find(|(k, _)| {
						let mut sequence_number = "seq_num".chars();
						k.iter().all(|k| Some(*k) == sequence_number.next())
					})
					.and_then(|v| match v.1 {
						JsonValue::Number(number) => Some(number),
						_ => None,
					})?
					.integer as u32;
				obj.into_iter()
					.find(|(k, _)| {
						let mut validators = "validators".chars();
						k.iter().all(|k| Some(*k) == validators.next())
					})
					.and_then(|(_, v)| match v {
						JsonValue::Array(vs) => {
							vs.iter().for_each(|v| match v {
								JsonValue::Object(obj) => {
									let id = obj
										.clone()
										.into_iter()
										.find(|(k, _)| {
											let mut id = "id".chars();
											k.iter().all(|k| Some(*k) == id.next())
										})
										.and_then(|v| match v.1 {
											JsonValue::String(s) => {
												let data: Vec<u8> = s
													.iter()
													.skip(2)
													.map(|c| *c as u8)
													.collect::<Vec<_>>();
												let b = hex::decode(data).map_err(|_| {
													log::info!("🐙 Not a valid hex string");
													Option::<ValidatorSet<<T as frame_system::Config>::AccountId>>::None
												}).ok()?;
												<T as frame_system::Config>::AccountId::decode(
													&mut &b[..],
												)
												.ok()
											}
											_ => None,
										});
									let weight = obj
										.clone()
										.into_iter()
										.find(|(k, _)| {
											let mut weight = "weight".chars();
											k.iter().all(|k| Some(*k) == weight.next())
										})
										.and_then(|v| match v.1 {
											JsonValue::Number(number) => Some(number),
											_ => None,
										});
									if id.is_some() && weight.is_some() {
										let id = id.expect("id is valid; qed");
										let weight =
											weight.expect("weight is valid; qed").integer
												as u64;
										val_set.validators.push(Validator {
											id: id,
											weight: weight,
										});
									}
								}
								_ => (),
							});
							Some(0)
						}
						_ => None,
					});
				Some(val_set)
			}
			_ => None,
		})
	}
}