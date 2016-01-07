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
pub fn gzip64res_to_json(source: &[u8]) -> Json {
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
fn json_to_rlp(json: &Json) -> Bytes {
	match json {
		&Json::Boolean(o) => encode(&(if o {1u64} else {0})),
		&Json::I64(o) => encode(&(o as u64)),
		&Json::U64(o) => encode(&o),
		&Json::String(ref s) if s.len() >= 2 && &s[0..2] == "0x" && U256::from_str(&s[2..]).is_ok() => {
			encode(&U256::from_str(&s[2..]).unwrap())
		},
		&Json::String(ref s) => {
			encode(s)
		},
		_ => panic!()
	}
}

/// Convert JSON to a string->RLP map.
fn json_to_rlp_map(json: &Json) -> HashMap<String, Bytes> {
	json.as_object().unwrap().iter().map(|(k, v)| (k, json_to_rlp(v))).fold(HashMap::new(), |mut acc, kv| {
		acc.insert(kv.0.clone(), kv.1);
		acc
	})
}

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
#[derive(Debug)]
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
			*self.state_root_memo.borrow_mut() = Some(sec_trie_root(self.genesis_state.iter().map(|(k, v)| (k.to_vec(), v.rlp())).collect()));
		}
		Ref::map(self.state_root_memo.borrow(), |x|x.as_ref().unwrap())
	}

	fn genesis_header(&self) -> Header {
		Header {
			parent_hash: self.parent_hash.clone(),
			timestamp: self.timestamp.clone(),
			number: U256::from(0u8),
			author: self.author.clone(),
			transactions_root: SHA3_NULL_RLP.clone(),
			uncles_hash: RlpStream::new_list(0).out().sha3(),
			extra_data: self.extra_data.clone(),
			state_root: self.state_root().clone(),
			receipts_root: SHA3_NULL_RLP.clone(),
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
		}
	}

	/// Compose the genesis block for this chain.
	pub fn genesis_block(&self) -> Bytes {
		let empty_list = RlpStream::new_list(0).out();
		let header = self.genesis_header();
		let mut ret = RlpStream::new_list(3);
		ret.append(&header);
		ret.append_raw(&empty_list, 1);
		ret.append_raw(&empty_list, 1);
		ret.out()
	}

	/// Loads a chain-specification from a json data structure
	pub fn from_json(json: Json) -> Spec {
		// once we commit ourselves to some json parsing library (serde?)
		// move it to proper data structure
		let mut state = HashMap::new();
		let mut builtins = HashMap::new();

		if let Some(&Json::Object(ref accounts)) = json.find("accounts") {
			for (address, acc) in accounts.iter() {
				let addr = Address::from_str(address).unwrap();
				if let Some(ref builtin_json) = acc.find("builtin") {
					if let Some(builtin) = Builtin::from_json(builtin_json) {
						builtins.insert(addr.clone(), builtin);
					}
				}
				let balance = if let Some(&Json::String(ref b)) = acc.find("balance") {U256::from_dec_str(b).unwrap_or(U256::from(0))} else {U256::from(0)};
				let nonce = if let Some(&Json::String(ref n)) = acc.find("nonce") {U256::from_dec_str(n).unwrap_or(U256::from(0))} else {U256::from(0)};
				// TODO: handle code & data if they exist.
				state.insert(addr, Account::new_basic(balance, nonce));
			}
		}

		let genesis = &json["genesis"];//.as_object().expect("No genesis object in JSON");

		let (seal_fields, seal_rlp) = {
			if genesis.find("mixHash").is_some() && genesis.find("nonce").is_some() {
				let mut s = RlpStream::new();
				s.append(&H256::from_str(&genesis["mixHash"].as_string().expect("mixHash not a string.")[2..]).expect("Invalid mixHash string value"));
				s.append(&H64::from_str(&genesis["nonce"].as_string().expect("nonce not a string.")[2..]).expect("Invalid nonce string value"));
				(2, s.out())
			} else {
				// backup algo that will work with sealFields/sealRlp (and without).
				(
					usize::from_str(&genesis["sealFields"].as_string().unwrap_or("0x")[2..]).expect("Invalid sealFields integer data"),
					genesis["sealRlp"].as_string().unwrap_or("0x")[2..].from_hex().expect("Invalid sealRlp hex data")
				)
			}
		};

		
		Spec {
			engine_name: json["engineName"].as_string().unwrap().to_string(),
			engine_params: json_to_rlp_map(&json["params"]),
			builtins: builtins,
			parent_hash: H256::from_str(&genesis["parentHash"].as_string().unwrap()[2..]).unwrap(),
			author: Address::from_str(&genesis["author"].as_string().unwrap()[2..]).unwrap(),
			difficulty: U256::from_str(&genesis["difficulty"].as_string().unwrap()[2..]).unwrap(),
			gas_limit: U256::from_str(&genesis["gasLimit"].as_string().unwrap()[2..]).unwrap(),
			gas_used: U256::from(0u8),
			timestamp: U256::from_str(&genesis["timestamp"].as_string().unwrap()[2..]).unwrap(),
			extra_data: genesis["extraData"].as_string().unwrap()[2..].from_hex().unwrap(),
			genesis_state: state,
			seal_fields: seal_fields,
			seal_rlp: seal_rlp,
			state_root_memo: RefCell::new(genesis.find("stateRoot").and_then(|_| genesis["stateRoot"].as_string()).map(|s| H256::from_str(&s[2..]).unwrap())),
		}
	}

	/// Returns the builtins map for the standard network of Ethereum Olympic, Frontier and Homestead.
	fn standard_builtins() -> HashMap<Address, Builtin> {
		let mut ret = HashMap::new();
		ret.insert(Address::from_str("0000000000000000000000000000000000000001").unwrap(), Builtin::from_named_linear("ecrecover", 3000, 0).unwrap());
		ret.insert(Address::from_str("0000000000000000000000000000000000000002").unwrap(), Builtin::from_named_linear("sha256", 60, 12).unwrap());
		ret.insert(Address::from_str("0000000000000000000000000000000000000003").unwrap(), Builtin::from_named_linear("ripemd160", 600, 120).unwrap());
		ret.insert(Address::from_str("0000000000000000000000000000000000000004").unwrap(), Builtin::from_named_linear("identity", 15, 3).unwrap());
		ret
	}

	/// Creates the Olympic network chain spec.
	pub fn new_like_olympic() -> Spec {
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
			builtins: Self::standard_builtins(),
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
			seal_rlp: { let mut r = RlpStream::new_list(2); r.append(&H256::new()); r.append(&0x2au64); r.out() },	// TODO: make correct
			state_root_memo: RefCell::new(None),
		}
	}

	/// Creates the Frontier network chain spec, except for the genesis state, which is blank.
	pub fn new_like_frontier() -> Spec {
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
			builtins: Self::standard_builtins(),
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
			seal_rlp: { let mut r = RlpStream::new_list(2); r.append(&H256::new()); r.append(&0x42u64); r.out() },
			state_root_memo: RefCell::new(None),
		}
	}

	/// Creates the actual Morden network chain spec.
	pub fn new_morden() -> Spec {
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
			builtins: Self::standard_builtins(),
			parent_hash: H256::new(),
			author: Address::new(),
			difficulty: U256::from(0x20000u64),
			gas_limit: U256::from(0x2fefd8u64),
			gas_used: U256::from(0u64),
			timestamp: U256::from(0u64),
			extra_data: vec![],
			genesis_state: {
				let n = U256::from(1) << 20;
				vec![
					(Address::from_str("0000000000000000000000000000000000000001").unwrap(), Account::new_basic(U256::from(1), n)),
					(Address::from_str("0000000000000000000000000000000000000002").unwrap(), Account::new_basic(U256::from(1), n)),
					(Address::from_str("0000000000000000000000000000000000000003").unwrap(), Account::new_basic(U256::from(1), n)),
					(Address::from_str("0000000000000000000000000000000000000004").unwrap(), Account::new_basic(U256::from(1), n)),
					(Address::from_str("102e61f5d8f9bc71d0ad4a084df4e65e05ce0e1c").unwrap(), Account::new_basic(U256::from(1) << 200, n))
				]}.into_iter().fold(HashMap::new(), | mut acc, vec | {
					acc.insert(vec.0, vec.1);
					acc
				}),
			seal_fields: 2,
			seal_rlp: {
				let mut r = RlpStream::new();
				r.append(&H256::from_str("00000000000000000000000000000000000000647572616c65787365646c6578").unwrap());
				r.append(&FromHex::from_hex("00006d6f7264656e").unwrap());
				r.out()
			},
			state_root_memo: RefCell::new(None),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::*;
	use util::sha3::*;
	use rustc_serialize::json::Json;
	use views::*;
	use super::*;

	#[test]
	fn all() {
		let morden = Spec::new_morden();
//		let engine = morden.to_engine();	// Ethash doesn't exist as an engine yet, so would fail.

		assert_eq!(*morden.state_root(), H256::from_str("f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9").unwrap());
		let genesis = morden.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303").unwrap());
	}

	#[test]
	fn morden_res() {
		let morden_json = Json::from_str(::std::str::from_utf8(include_bytes!("../res/morden.json")).unwrap()).expect("Json is invalid");
		let morden = Spec::from_json(morden_json);

//		let engine = morden.to_engine();	// Ethash doesn't exist as an engine yet, so would fail.

		assert_eq!(*morden.state_root(), H256::from_str("f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9").unwrap());
		let genesis = morden.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303").unwrap());
	}
}