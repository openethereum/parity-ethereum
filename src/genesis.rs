use std::io::Read;
use std::str::FromStr;
use std::collections::HashMap;
use rustc_serialize::base64::FromBase64;
use rustc_serialize::json::Json;
use rustc_serialize::hex::FromHex;
use flate2::read::GzDecoder;
use util::rlp::*;
use util::hash::*;
use util::uint::*;
use util::sha3::*;
use account::*;
use header::*;

/// Converts file from base64 gzipped bytes to json
fn base_to_json(source: &[u8]) -> Json {
	// there is probably no need to store genesis in based64 gzip,
	// but that's what go does, and it was easy to load it this way
	let data = source.from_base64().expect("Genesis block is malformed!");
	let data_ref: &[u8] = &data;
	let mut decoder = GzDecoder::new(data_ref).expect("Gzip is invalid");
	let mut s: String = "".to_string();
	decoder.read_to_string(&mut s).expect("Gzip is invalid");
	Json::from_str(&s).expect("Json is invalid")
}

pub struct Genesis {
	block: Vec<u8>,
	state: HashMap<Address, Account>
}

impl Genesis {
	/// Creates genesis block for frontier network
	pub fn new_frontier() -> Genesis {
		let root = H256::from_str("d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544").unwrap();
		let json = base_to_json(include_bytes!("../res/genesis_frontier"));
		let (header, state) = Self::load_genesis_json(json, root);
		Self::new_from_header_and_state(header, state)
	}

	/// Creates genesis block from header and state hashmap
	pub fn new_from_header_and_state(header: Header, state: HashMap<Address, Account>) -> Genesis {
		let empty_list = RlpStream::new_list(0).out();
		let mut stream = RlpStream::new_list(3);
		stream.append(&header);
		stream.append_raw(&empty_list, 1);
		stream.append_raw(&empty_list, 1);

		Genesis {
			block: stream.out(),
			state: state 
		}
	}

	/// Loads genesis block from json file
	fn load_genesis_json(json: Json, state_root: H256) -> (Header, HashMap<Address, Account>)  {
		// once we commit ourselves to some json parsing library (serde?)
		// move it to proper data structure
		
		let empty_list = RlpStream::new_list(0).out();
		let empty_list_sha3 = empty_list.sha3();
		let empty_data = encode(&"");
		let empty_data_sha3 = empty_data.sha3();
		
		let mut state = HashMap::new();
		let accounts = json["alloc"].as_object().expect("Missing genesis state");
		for (address, acc) in accounts.iter() {
			let addr = Address::from_str(address).unwrap();
			let o = acc.as_object().unwrap();
			let balance = U256::from_dec_str(o["balance"].as_string().unwrap()).unwrap();
			state.insert(addr, Account::new_basic(balance, U256::from(0)));
		}

		let header = Header {
			parent_hash: H256::from_str(&json["parentHash"].as_string().unwrap()[2..]).unwrap(),
			uncles_hash: empty_list_sha3.clone(),
			author: Address::from_str(&json["coinbase"].as_string().unwrap()[2..]).unwrap(),
			state_root: state_root,
			transactions_root: empty_data_sha3.clone(),
			receipts_root: empty_data_sha3.clone(),
			log_bloom: H2048::new(),
			difficulty: U256::from_str(&json["difficulty"].as_string().unwrap()[2..]).unwrap(),
			number: U256::from(0u8),
			gas_limit: U256::from_str(&json["gasLimit"].as_string().unwrap()[2..]).unwrap(),
			gas_used: U256::from(0u8),
			timestamp: U256::from_str(&json["timestamp"].as_string().unwrap()[2..]).unwrap(),
			extra_data: json["extraData"].as_string().unwrap()[2..].from_hex().unwrap(),
			seal: {
				// ethash specific fields
				let mixhash = H256::from_str(&json["mixhash"].as_string().unwrap()[2..]).unwrap();
				let nonce = H64::from_str(&json["nonce"].as_string().unwrap()[2..]).unwrap();
				vec![mixhash.to_vec(), nonce.to_vec()]
			}
		};
		
		(header, state)
	}

	/// Returns genesis block
	pub fn block(&self) -> &[u8] {
		&self.block
	}

	/// Returns genesis block state
	pub fn state(&self) -> &HashMap<Address, Account> {
		&self.state
	}

	// not sure if this one is needed
	pub fn drain(self) -> (Vec<u8>, HashMap<Address, Account>) {
		(self.block, self.state)
	}
}

#[test]
fn test_genesis() {
	use views::*;

	let g = Genesis::new_frontier();
	let view = BlockView::new(&g.block).header_view();
	let genesis_hash = H256::from_str("347db3ae87cf4703f948676de5858af1a2a336cbe2e6e56c5041dd80bed3071f").unwrap();
	assert_eq!(view.sha3(), genesis_hash);
}
