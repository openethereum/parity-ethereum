use std::io::Read;
use std::collections::HashMap;
use std::cell::*;
use std::str::FromStr;
use rustc_serialize::base64::FromBase64;
use rustc_serialize::json::Json;
use rustc_serialize::hex::FromHex;
use flate2::read::GzDecoder;
use util::uint::*;
use util::hash::*;
use util::bytes::*;
use util::triehash::*;
use util::error::*;
use util::rlp::*;
use util::sha3::*;
use account::*;
use engine::Engine;
use builtin::Builtin;
use null_engine::NullEngine;
use denominations::*;
use header::*;

/// Converts file from base64 gzipped bytes to json
pub fn base_to_json(source: &[u8]) -> Json {
	// there is probably no need to store genesis in based64 gzip,
	// but that's what go does, and it was easy to load it this way
	let data = source.from_base64().expect("Genesis block is malformed!");
	let data_ref: &[u8] = &data;
	let mut decoder = GzDecoder::new(data_ref).expect("Gzip is invalid");
	let mut s: String = "".to_string();
	decoder.read_to_string(&mut s).expect("Gzip is invalid");
	Json::from_str(&s).expect("Json is invalid")
}

/// Convert JSON value to equivlaent RLP representation.
// TODO: handle container types.
pub fn json_to_rlp(json: &Json) -> Bytes {
	match json {
		&Json::I64(o) => encode(&(o as u64)),
		&Json::U64(o) => encode(&o),
		&Json::String(ref s) if &s[0..2] == "0x" && U256::from_str(&s[2..]).is_ok() => {
			encode(&U256::from_str(&s[2..]).unwrap())
		},
		&Json::String(ref s) => {
			encode(s)
		},
		_ => panic!()
	}
}

/// Convert JSON to a string->RLP map.
pub fn json_to_rlp_map(json: &Json) -> HashMap<String, Bytes> {
	json.as_object().unwrap().iter().map(|(k, v)| (k, json_to_rlp(v))).fold(HashMap::new(), |mut acc, kv| {
				acc.insert(kv.0.clone(), kv.1);
				acc
			})
}

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Spec {
	// What engine are we using for this?
	pub engine_name: String,

	// Parameters concerning operation of the specific engine we're using.
	// Name -> RLP-encoded value
	pub engine_params: HashMap<String, Bytes>,

	// Builtin-contracts are here for now but would like to abstract into Engine API eventually.
	pub builtins: HashMap<Address, Builtin>,

	// Genesis params.
	pub parent_hash: H256,
	pub author: Address,
	pub difficulty: U256,
	pub gas_limit: U256,
	pub gas_used: U256,
	pub timestamp: U256,
	pub extra_data: Bytes,
	pub genesis_state: HashMap<Address, Account>,
	pub seal_fields: usize,
	pub seal_rlp: Bytes,

	// May be prepopulated if we know this in advance.
	state_root_memo: RefCell<Option<H256>>,
}

impl Spec {
	/// Convert this object into a boxed Engine of the right underlying type.
	// TODO avoid this hard-coded nastiness - use dynamic-linked plugin framework instead.
	pub fn to_engine(self) -> Result<Box<Engine>, EthcoreError> {
		match self.engine_name.as_ref() {
			"NullEngine" => Ok(NullEngine::new_boxed(self)),
			_ => Err(EthcoreError::UnknownName)
		}
	}

	/// Return the state root for the genesis state, memoising accordingly.
	pub fn state_root(&self) -> Ref<H256> {
		if self.state_root_memo.borrow().is_none() {
			*self.state_root_memo.borrow_mut() = Some(trie_root(self.genesis_state.iter().map(|(k, v)| (k.to_vec(), v.rlp())).collect()));
		}
		Ref::map(self.state_root_memo.borrow(), |x|x.as_ref().unwrap())
	}

	/// Compose the genesis block for this chain.
	pub fn genesis_block(&self) -> Bytes {
		let empty_list = RlpStream::new_list(0).out();
		let empty_list_sha3 = empty_list.sha3();
		let header = Header {
			parent_hash: self.parent_hash.clone(),
			timestamp: self.timestamp.clone(),
			number: U256::from(0u8),
			author: self.author.clone(),
			transactions_root: SHA3_EMPTY.clone(),
			uncles_hash: empty_list_sha3.clone(),
			extra_data: self.extra_data.clone(),
			state_root: self.state_root().clone(),
			receipts_root: SHA3_EMPTY.clone(),
			log_bloom: H2048::new().clone(),
			gas_used: self.gas_used.clone(),
			gas_limit: self.gas_limit.clone(),
			difficulty: self.difficulty.clone(),
			seal: {
				let seal = {
					let mut s = RlpStream::new_list(self.seal_fields);
					s.append_raw(&self.seal_rlp, self.seal_fields);
					s.out()
				};
				let r = Rlp::new(&seal);
				(0..self.seal_fields).map(|i| r.at(i).raw().to_vec()).collect()
			},
			hash: RefCell::new(None)
		};
		let mut ret = RlpStream::new_list(3);
		ret.append(&header);
		ret.append_raw(&empty_list, 1);
		ret.append_raw(&empty_list, 1);
		ret.out()
	}
}


impl Spec {
	/// Loads a chain-specification from a json data structure
	pub fn from_json(json: Json) -> Spec {
		// once we commit ourselves to some json parsing library (serde?)
		// move it to proper data structure
		let mut state = HashMap::new();
		let accounts = json["alloc"].as_object().expect("Missing genesis state");
		for (address, acc) in accounts.iter() {
			let addr = Address::from_str(address).unwrap();
			let o = acc.as_object().unwrap();
			let balance = U256::from_dec_str(o["balance"].as_string().unwrap()).unwrap();
			state.insert(addr, Account::new_basic(balance, U256::from(0)));
		}

		let builtins = {
			// TODO: populate from json.
			HashMap::new()
		};

		let (seal_fields, seal_rlp) = {
			if json.find("mixhash").is_some() && json.find("nonce").is_some() {
				let mut s = RlpStream::new();
				s.append(&H256::from_str(&json["mixhash"].as_string().unwrap()[2..]).unwrap());
				s.append(&H64::from_str(&json["nonce"].as_string().unwrap()[2..]).unwrap());
				(2, s.out())
			} else {
				// backup algo that will work with sealFields/sealRlp (and without).
				(usize::from_str(&json["sealFields"].as_string().unwrap_or("0x")[2..]).unwrap(), json["sealRlp"].as_string().unwrap_or("0x")[2..].from_hex().unwrap())
			}
		};

		Spec {
			engine_name: json["engineName"].as_string().unwrap().to_string(),
			engine_params: json_to_rlp_map(&json["params"]),
			builtins: builtins,
			parent_hash: H256::from_str(&json["parentHash"].as_string().unwrap()[2..]).unwrap(),
			author: Address::from_str(&json["coinbase"].as_string().unwrap()[2..]).unwrap(),
			difficulty: U256::from_str(&json["difficulty"].as_string().unwrap()[2..]).unwrap(),
			gas_limit: U256::from_str(&json["gasLimit"].as_string().unwrap()[2..]).unwrap(),
			gas_used: U256::from(0u8),
			timestamp: U256::from_str(&json["timestamp"].as_string().unwrap()[2..]).unwrap(),
			extra_data: json["extraData"].as_string().unwrap()[2..].from_hex().unwrap(),
			genesis_state: state,
			seal_fields: seal_fields,
			seal_rlp: seal_rlp,
			state_root_memo: RefCell::new(json["stateRoot"].as_string().map(|s| H256::from_str(&s[2..]).unwrap())),
		}
	}

	/// Creates the Olympic network chain spec.
	pub fn olympic() -> Spec {
		Spec {
			engine_name: "Ethash".to_string(),
			engine_params: vec![
				("block_reward", encode(&(finney() * U256::from(1500u64)))),
				("maximum_extra_data_size", encode(&U256::from(1024u64))),
				("account_start_nonce", encode(&U256::from(0u64))),
				("gas_limit_bounds_divisor", encode(&1024u64)),
				("minimum_difficulty", encode(&131_072u64)),
				("difficulty_bound_divisor", encode(&2048u64)),
				("duration_limit", encode(&8u64)),
				("min_gas_limit", encode(&125_000u64)),
				("gas_floor_target", encode(&3_141_592u64)),
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0.to_string(), vec.1);
				acc
			}),
			builtins: HashMap::new(),			// TODO: make correct
			parent_hash: H256::new(),
			author: Address::new(),
			difficulty: U256::from(131_072u64),
			gas_limit: U256::from(0u64),
			gas_used: U256::from(0u64),
			timestamp: U256::from(0u64),
			extra_data: vec![],
			genesis_state: vec![				// TODO: make correct
				(Address::new(), Account::new_basic(U256::from(1) << 200, U256::from(0)))
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0, vec.1);
				acc
			}),
			seal_fields: 2,
			seal_rlp: { let mut r = RlpStream::new_list(2); r.append(&0x2au64); r.append(&H256::new()); r.out() },	// TODO: make correct
			state_root_memo: RefCell::new(None),
		}
	}

	/// Creates the Frontier network chain spec.
	pub fn frontier() -> Spec {
		Spec {
			engine_name: "Ethash".to_string(),
			engine_params: vec![
				("block_reward", encode(&(ether() * U256::from(5u64)))),
				("maximum_extra_data_size", encode(&U256::from(32u64))),
				("account_start_nonce", encode(&U256::from(0u64))),
				("gas_limit_bounds_divisor", encode(&1024u64)),
				("minimum_difficulty", encode(&131_072u64)),
				("difficulty_bound_divisor", encode(&2048u64)),
				("duration_limit", encode(&13u64)),
				("min_gas_limit", encode(&5000u64)),
				("gas_floor_target", encode(&3_141_592u64)),
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0.to_string(), vec.1);
				acc
			}),
			builtins: HashMap::new(),			// TODO: make correct
			parent_hash: H256::new(),
			author: Address::new(),
			difficulty: U256::from(131_072u64),
			gas_limit: U256::from(0u64),
			gas_used: U256::from(0u64),
			timestamp: U256::from(0u64),
			extra_data: vec![],
			genesis_state: vec![				// TODO: make correct
				(Address::new(), Account::new_basic(U256::from(1) << 200, U256::from(0)))
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0, vec.1);
				acc
			}),
			seal_fields: 2,
			seal_rlp: { let mut r = RlpStream::new_list(2); r.append(&0x42u64); r.append(&H256::new()); r.out() },
			state_root_memo: RefCell::new(None),
		}
	}

	/// Creates the Morden network chain spec.
	pub fn morden() -> Spec {
		Spec {
			engine_name: "Ethash".to_string(),
			engine_params: vec![
				("block_reward", encode(&(ether() * U256::from(5u64)))),
				("maximum_extra_data_size", encode(&U256::from(32u64))),
				("account_start_nonce", encode(&(U256::from(1u64) << 20))),
				("gas_limit_bounds_divisor", encode(&1024u64)),
				("minimum_difficulty", encode(&131_072u64)),
				("difficulty_bound_divisor", encode(&2048u64)),
				("duration_limit", encode(&13u64)),
				("min_gas_limit", encode(&5000u64)),
				("gas_floor_target", encode(&3_141_592u64)),
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0.to_string(), vec.1);
				acc
			}),
			builtins: HashMap::new(),			// TODO: make correct
			parent_hash: H256::new(),
			author: Address::new(),
			difficulty: U256::from(131_072u64),
			gas_limit: U256::from(0u64),
			gas_used: U256::from(0u64),
			timestamp: U256::from(0u64),
			extra_data: vec![],
			genesis_state: vec![				// TODO: make correct
				(Address::new(), Account::new_basic(U256::from(1) << 200, U256::from(0)))
			].into_iter().fold(HashMap::new(), | mut acc, vec | {
				acc.insert(vec.0, vec.1);
				acc
			}),
			seal_fields: 2,
			seal_rlp: { let mut r = RlpStream::new_list(2); r.append(&0x00006d6f7264656eu64); r.append(&H256::from_str("00000000000000000000000000000000000000647572616c65787365646c6578").unwrap()); r.out() },	// TODO: make correct
			state_root_memo: RefCell::new(None),
		}
	}
}

