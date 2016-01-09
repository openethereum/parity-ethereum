use common::*;
use flate2::read::GzDecoder;
use engine::*;
use null_engine::*;

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
			"Ethash" => Ok(super::ethereum::Ethash::new_boxed(self)),
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

	pub fn genesis_header(&self) -> Header {
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
				(0..self.seal_fields).map(|i| r.at(i).as_raw().to_vec()).collect()
			},
			hash: RefCell::new(None),
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
				let balance = acc.find("balance").and_then(|x| match x { &Json::String(ref b) => U256::from_dec_str(b).ok(), _ => None });
				let nonce = acc.find("nonce").and_then(|x| match x { &Json::String(ref b) => U256::from_dec_str(b).ok(), _ => None });
//				let balance = if let Some(&Json::String(ref b)) = acc.find("balance") {U256::from_dec_str(b).unwrap_or(U256::from(0))} else {U256::from(0)};
//				let nonce = if let Some(&Json::String(ref n)) = acc.find("nonce") {U256::from_dec_str(n).unwrap_or(U256::from(0))} else {U256::from(0)};
				// TODO: handle code & data if they exist.
				if balance.is_some() || nonce.is_some() {
					state.insert(addr, Account::new_basic(balance.unwrap_or(U256::from(0)), nonce.unwrap_or(U256::from(0))));
				}
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

	/// Ensure that the given state DB has the trie nodes in for the genesis state.
	pub fn ensure_db_good(&self, db: &mut HashDB) {
		if !db.contains(&self.state_root()) {
			let mut root = H256::new();
			let mut t = SecTrieDBMut::new(db, &mut root);
			for (address, account) in self.genesis_state.iter() {
				t.insert(address.as_slice(), &account.rlp());
			}
		}
	}

	/// Create a new Spec from a JSON UTF-8 data resource `data`.
	pub fn from_json_utf8(data: &[u8]) -> Spec {
		Self::from_json_str(::std::str::from_utf8(data).unwrap())
	}

	/// Create a new Spec from a JSON string.
	pub fn from_json_str(s: &str) -> Spec {
		let json = Json::from_str(s).expect("Json is invalid");
		Self::from_json(json)
	}

	/// Create a new Olympic chain spec.
	pub fn new_olympic() -> Spec { Self::from_json_utf8(include_bytes!("../res/olympic.json")) }

	/// Create a new Frontier chain spec.
	pub fn new_frontier() -> Spec { Self::from_json_utf8(include_bytes!("../res/frontier.json")) }

	/// Create a new Morden chain spec.
	pub fn new_morden() -> Spec { Self::from_json_utf8(include_bytes!("../res/morden.json")) }
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::*;
	use util::sha3::*;
	use views::*;
	use super::*;

	#[test]
	fn morden() {
		let morden = Spec::new_morden();

		assert_eq!(*morden.state_root(), H256::from_str("f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9").unwrap());
		let genesis = morden.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303").unwrap());

		morden.to_engine();
	}

	#[test]
	fn frontier() {
		let frontier = Spec::new_frontier();

		assert_eq!(*frontier.state_root(), H256::from_str("d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544").unwrap());
		let genesis = frontier.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap());

		frontier.to_engine();
	}
}