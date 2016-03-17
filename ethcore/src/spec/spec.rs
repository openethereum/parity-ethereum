// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Parameters for a block chain.

use common::*;
use engine::*;
use pod_state::*;
use null_engine::*;
use account_db::*;
use ethereum;
use super::genesis::{Seal as GenesisSeal, Genesis};

/// Convert JSON value to equivalent RLP representation.
// TODO: handle container types.
fn json_to_rlp(json: &Json) -> Bytes {
	match *json {
		Json::Boolean(o) => encode(&(if o {1u64} else {0})).to_vec(),
		Json::I64(o) => encode(&(o as u64)).to_vec(),
		Json::U64(o) => encode(&o).to_vec(),
		Json::String(ref s) if s.len() >= 2 && &s[0..2] == "0x" && U256::from_str(&s[2..]).is_ok() => {
			encode(&U256::from_str(&s[2..]).unwrap()).to_vec()
		},
		Json::String(ref s) => {
			encode(s).to_vec()
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
	/// User friendly spec name
	pub name: String,
	/// What engine are we using for this?
	pub engine_name: String,

	/// Known nodes on the network in enode format.
	pub nodes: Vec<String>,
	/// Network ID
	pub network_id: U256,

	/// Parameters concerning operation of the specific engine we're using.
	/// Maps the parameter name to an RLP-encoded value.
	pub engine_params: HashMap<String, Bytes>,

	/// Builtin-contracts we would like to see in the chain.
	/// (In principle these are just hints for the engine since that has the last word on them.)
	pub builtins: BTreeMap<Address, Builtin>,

	/// The genesis block's parent hash field.
	pub parent_hash: H256,
	/// The genesis block's author field.
	pub author: Address,
	/// The genesis block's difficulty field.
	pub difficulty: U256,
	/// The genesis block's gas limit field.
	pub gas_limit: U256,
	/// The genesis block's gas used field.
	pub gas_used: U256,
	/// The genesis block's timestamp field.
	pub timestamp: u64,
	/// Transactions root of the genesis block. Should be SHA3_NULL_RLP.
	pub transactions_root: H256,
	/// Receipts root of the genesis block. Should be SHA3_NULL_RLP.
	pub receipts_root: H256,
	/// The genesis block's extra data field.
	pub extra_data: Bytes,
	/// The number of seal fields in the genesis block.
	pub seal_fields: usize,
	/// Each seal field, expressed as RLP, concatenated.
	pub seal_rlp: Bytes,

	// May be prepopulated if we know this in advance.
	state_root_memo: RwLock<Option<H256>>,

	// Genesis state as plain old data.
	genesis_state: PodState,
}

#[cfg_attr(feature="dev", allow(wrong_self_convention))] // because to_engine(self) should be to_engine(&self)
impl Spec {
	/// Convert this object into a boxed Engine of the right underlying type.
	// TODO avoid this hard-coded nastiness - use dynamic-linked plugin framework instead.
	pub fn to_engine(self) -> Result<Box<Engine>, Error> {
		match self.engine_name.as_ref() {
			"NullEngine" => Ok(NullEngine::new_boxed(self)),
			"Ethash" => Ok(ethereum::Ethash::new_boxed(self)),
			_ => Err(Error::UnknownEngineName(self.engine_name.clone()))
		}
	}

	/// Return the state root for the genesis state, memoising accordingly.
	pub fn state_root(&self) -> H256 {
		if self.state_root_memo.read().unwrap().is_none() {
			*self.state_root_memo.write().unwrap() = Some(self.genesis_state.root());
		}
		self.state_root_memo.read().unwrap().as_ref().unwrap().clone()
	}

	/// Get the known knodes of the network in enode format.
	pub fn nodes(&self) -> &Vec<String> { &self.nodes }

	/// Get the configured Network ID.
	pub fn network_id(&self) -> U256 { self.network_id }

	/// Get the header of the genesis block.
	pub fn genesis_header(&self) -> Header {
		Header {
			parent_hash: self.parent_hash.clone(),
			timestamp: self.timestamp,
			number: 0,
			author: self.author.clone(),
			transactions_root: self.transactions_root.clone(),
			uncles_hash: RlpStream::new_list(0).out().sha3(),
			extra_data: self.extra_data.clone(),
			state_root: self.state_root().clone(),
			receipts_root: self.receipts_root.clone(),
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
			bare_hash: RefCell::new(None),
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

	/// Overwrite the genesis components with the given JSON, assuming standard Ethereum test format.
	pub fn overwrite_genesis(&mut self, genesis: &Json) {
		let (seal_fields, seal_rlp) = {
			if genesis.find("mixHash").is_some() && genesis.find("nonce").is_some() {
				let mut s = RlpStream::new();
				s.append(&H256::from_json(&genesis["mixHash"]));
				s.append(&H64::from_json(&genesis["nonce"]));
				(2, s.out())
			} else {
				// backup algo that will work with sealFields/sealRlp (and without).
				(
					u64::from_json(&genesis["sealFields"]) as usize,
					Bytes::from_json(&genesis["sealRlp"])
				)
			}
		};

		self.parent_hash = H256::from_json(&genesis["parentHash"]);
		self.transactions_root = genesis.find("transactionsTrie").and_then(|_| Some(H256::from_json(&genesis["transactionsTrie"]))).unwrap_or(SHA3_NULL_RLP.clone());
		self.receipts_root = genesis.find("receiptTrie").and_then(|_| Some(H256::from_json(&genesis["receiptTrie"]))).unwrap_or(SHA3_NULL_RLP.clone());
		self.author = Address::from_json(&genesis["coinbase"]);
		self.difficulty = U256::from_json(&genesis["difficulty"]);
		self.gas_limit = U256::from_json(&genesis["gasLimit"]);
		self.gas_used = U256::from_json(&genesis["gasUsed"]);
		self.timestamp = u64::from_json(&genesis["timestamp"]);
		self.extra_data = Bytes::from_json(&genesis["extraData"]);
		self.seal_fields = seal_fields;
		self.seal_rlp = seal_rlp;
		self.state_root_memo = RwLock::new(genesis.find("stateRoot").and_then(|_| Some(H256::from_json(&genesis["stateRoot"]))));
	}

	/// Overwrite the genesis components.
	pub fn overwrite_genesis_params(&mut self, g: Genesis) {
		let (seal_fields, seal_rlp) = match g.seal {
			GenesisSeal::Generic { fields, rlp } => (fields, rlp),
			GenesisSeal::Ethereum { nonce, mix_hash } => {
				let mut s = RlpStream::new();
				s.append(&mix_hash);
				s.append(&nonce);
				(2, s.out())
			}
		};

		self.parent_hash = g.parent_hash;
		self.transactions_root = g.transactions_root;
		self.receipts_root = g.receipts_root;
		self.author = g.author;
		self.difficulty = g.difficulty;
		self.gas_limit = g.gas_limit;
		self.gas_used = g.gas_used;
		self.timestamp = g.timestamp;
		self.extra_data = g.extra_data;
		self.seal_fields = seal_fields;
		self.seal_rlp = seal_rlp;
		self.state_root_memo = RwLock::new(g.state_root);
	}

	/// Alter the value of the genesis state.
	pub fn set_genesis_state(&mut self, s: PodState) {
		self.genesis_state = s;
		*self.state_root_memo.write().unwrap() = None;
	}

	/// Returns `false` if the memoized state root is invalid. `true` otherwise.
	pub fn is_state_root_valid(&self) -> bool {
		self.state_root_memo.read().unwrap().clone().map_or(true, |sr| sr == self.genesis_state.root())
	}
}

impl FromJson for Spec {
	/// Loads a chain-specification from a json data structure
	fn from_json(json: &Json) -> Spec {
		// once we commit ourselves to some json parsing library (serde?)
		// move it to proper data structure
		let mut builtins = BTreeMap::new();
		let mut state = PodState::new();

		if let Some(&Json::Object(ref accounts)) = json.find("accounts") {
			for (address, acc) in accounts.iter() {
				let addr = Address::from_str(address).unwrap();
				if let Some(ref builtin_json) = acc.find("builtin") {
					if let Some(builtin) = Builtin::from_json(builtin_json) {
						builtins.insert(addr.clone(), builtin);
					}
				}
			}
			state = xjson!(&json["accounts"]);
		}

		let nodes = if let Some(&Json::Array(ref ns)) = json.find("nodes") {
			ns.iter().filter_map(|n| if let Json::String(ref s) = *n { Some(s.clone()) } else {None}).collect()
		} else { Vec::new() };

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
			name: json.find("name").map_or("unknown", |j| j.as_string().unwrap()).to_owned(),
			engine_name: json["engineName"].as_string().unwrap().to_owned(),
			engine_params: json_to_rlp_map(&json["params"]),
			nodes: nodes,
			network_id: U256::from_str(&json["params"]["networkID"].as_string().unwrap()[2..]).unwrap(),
			builtins: builtins,
			parent_hash: H256::from_str(&genesis["parentHash"].as_string().unwrap()[2..]).unwrap(),
			author: Address::from_str(&genesis["author"].as_string().unwrap()[2..]).unwrap(),
			difficulty: U256::from_str(&genesis["difficulty"].as_string().unwrap()[2..]).unwrap(),
			gas_limit: U256::from_str(&genesis["gasLimit"].as_string().unwrap()[2..]).unwrap(),
			gas_used: U256::from(0u8),
			timestamp: u64::from_str(&genesis["timestamp"].as_string().unwrap()[2..]).unwrap(),
			transactions_root: SHA3_NULL_RLP.clone(),
			receipts_root: SHA3_NULL_RLP.clone(),
			extra_data: genesis["extraData"].as_string().unwrap()[2..].from_hex().unwrap(),
			genesis_state: state,
			seal_fields: seal_fields,
			seal_rlp: seal_rlp,
			state_root_memo: RwLock::new(genesis.find("stateRoot").and_then(|_| genesis["stateRoot"].as_string()).map(|s| H256::from_str(&s[2..]).unwrap())),
		}
	}
}

impl Spec {
	/// Ensure that the given state DB has the trie nodes in for the genesis state.
	pub fn ensure_db_good(&self, db: &mut HashDB) -> bool {
		if !db.contains(&self.state_root()) {
			let mut root = H256::new();
			{
				let mut t = SecTrieDBMut::new(db, &mut root);
				for (address, account) in self.genesis_state.get().iter() {
					t.insert(address.as_slice(), &account.rlp());
				}
			}
			for (address, account) in self.genesis_state.get().iter() {
				account.insert_additional(&mut AccountDBMut::new(db, address));
			}
			assert!(db.contains(&self.state_root()));
			true
		} else { false }
	}

	/// Create a new Spec from a JSON UTF-8 data resource `data`.
	pub fn from_json_utf8(data: &[u8]) -> Spec {
		Self::from_json_str(::std::str::from_utf8(data).unwrap())
	}

	/// Create a new Spec from a JSON string.
	pub fn from_json_str(s: &str) -> Spec {
		Self::from_json(&Json::from_str(s).expect("Json is invalid"))
	}

	/// Create a new Spec which conforms to the Morden chain except that it's a NullEngine consensus.
	pub fn new_test() -> Spec { Self::from_json_utf8(include_bytes!("../../res/null_morden.json")) }
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::*;
	use util::sha3::*;
	use views::*;
	use super::*;

	#[test]
	fn test_chain() {
		let test_spec = Spec::new_test();

		assert_eq!(test_spec.state_root(), H256::from_str("f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9").unwrap());
		let genesis = test_spec.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303").unwrap());

		let _ = test_spec.to_engine();
	}
}
