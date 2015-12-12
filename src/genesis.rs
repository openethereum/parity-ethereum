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
		Self::new_from_json(&json, &root)
	}

	/// Loads genesis block from json file
	pub fn new_from_json(json: &Json, state_root: &H256) -> Genesis {
		// once we commit ourselves to some json parsing library (serde?)
		// move it to proper data structure
		let mixhash = H256::from_str(&json["mixhash"].as_string().unwrap()[2..]).unwrap();
		let parent_hash = H256::from_str(&json["parentHash"].as_string().unwrap()[2..]).unwrap();
		let coinbase = Address::from_str(&json["coinbase"].as_string().unwrap()[2..]).unwrap();
		let difficulty = U256::from_str(&json["difficulty"].as_string().unwrap()[2..]).unwrap();
		let gas_limit = U256::from_str(&json["gasLimit"].as_string().unwrap()[2..]).unwrap();
		let timestamp = U256::from_str(&json["timestamp"].as_string().unwrap()[2..]).unwrap();
		let extra_data: Vec<u8> = json["extraData"].as_string().unwrap()[2..].from_hex().unwrap();
		let nonce = H64::from_str(&json["nonce"].as_string().unwrap()[2..]).unwrap();

		let log_bloom = H2048::new();
		let number = 0u16;
		let gas_used = 0u16;

		let empty_list = RlpStream::new_list(0).out();
		let empty_list_sha3 = empty_list.sha3();
		let empty_data = encode(&"");
		let empty_data_sha3 = empty_data.sha3();

		let mut stream = RlpStream::new_list(3);
		stream.append_list(15);
		stream.append(&parent_hash);
		// uncles - empty list sha3
		stream.append(&empty_list_sha3);
		stream.append(&coinbase);
		stream.append(state_root);
		// transactions
		stream.append(&empty_data_sha3);
		// receipts
		stream.append(&empty_data_sha3);
		stream.append(&log_bloom);
		stream.append(&difficulty);
		stream.append(&number);
		stream.append(&gas_limit);
		stream.append(&gas_used);
		stream.append(&timestamp);
		stream.append(&extra_data);
		stream.append(&mixhash);
		stream.append(&nonce);
		stream.append_raw(&empty_list, 1);
		stream.append_raw(&empty_list, 1);
	
		let mut state = HashMap::new();
		let accounts = json["alloc"].as_object().expect("Missing genesis state");
		for (address, acc) in accounts.iter() {
			let addr = Address::from_str(address).unwrap();
			let o = acc.as_object().unwrap();
			let balance = U256::from_dec_str(o["balance"].as_string().unwrap()).unwrap();
			state.insert(addr, Account::new_with_balance(balance));
		}

		Genesis {
			block: stream.out(),
			state: state 
		}
	}

	pub fn drain(self) -> (Vec<u8>, HashMap<Address, Account>) {
		(self.block, self.state)
	}
}

#[test]
fn test_genesis() {
	use blockheader::*;

	let g = Genesis::new_frontier();
	let view = BlockView::new_from_rlp(Rlp::new(&g.block).at(0));
	let genesis_hash = H256::from_str("d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap();
	assert_eq!(view.sha3(), genesis_hash);
}
