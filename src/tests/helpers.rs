use client::{BlockChainClient,Client};
use std::env;
use super::test_common::*;
use std::path::PathBuf;
use spec::*;
use std::fs::{remove_dir_all};


pub struct RandomTempPath {
	path: PathBuf
}

impl RandomTempPath {
	pub fn new() -> RandomTempPath {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		RandomTempPath {
			path: dir.clone()
		}
	}

	pub fn as_path(&self) -> &PathBuf {
		&self.path
	}
}

impl Drop for RandomTempPath {
	fn drop(&mut self) {
		if let Err(e) = remove_dir_all(self.as_path()) {
			panic!("failed to remove temp directory, probably something failed to destroyed ({})", e);
		}
	}
}

pub fn get_test_spec() -> Spec {
	Spec::new_test()
}

pub fn create_test_block(header: &Header) -> Bytes {
	let mut rlp = RlpStream::new_list(3);
	rlp.append(header);
	rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
	rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
	rlp.out()
}

pub fn create_test_block_with_data(header: &Header, transactions: &[&Transaction], uncles: &[Header]) -> Bytes {
	let mut rlp = RlpStream::new_list(3);
	rlp.append(header);
	rlp.append_list(transactions.len());
	for t in transactions {
		rlp.append_raw(&t.rlp_bytes_opt(Seal::With), 1);
	}
	rlp.append_list(uncles.len());
	for h in uncles {
		rlp.append(h);
	}
	rlp.out()
}


pub fn generate_dummy_client(block_number: usize) -> Arc<Client> {
	let dir = RandomTempPath::new();

	let client = Client::new(get_test_spec(), dir.as_path(), IoChannel::disconnected()).unwrap();
	let test_spec = get_test_spec();
	let test_engine = test_spec.to_engine().unwrap();
	let state_root = test_engine.spec().genesis_header().state_root;
	let mut rolling_hash = test_engine.spec().genesis_header().hash();
	let mut rolling_block_number = 1;
	let mut rolling_timestamp = 40;

	for _ in 0..block_number {
		let mut header = Header::new();

		header.gas_limit = decode(test_engine.spec().engine_params.get("minGasLimit").unwrap());
		header.difficulty = decode(test_engine.spec().engine_params.get("minimumDifficulty").unwrap());
		header.timestamp = rolling_timestamp;
		header.number = rolling_block_number;
		header.parent_hash = rolling_hash;
		header.state_root = state_root.clone();

		rolling_hash = header.hash();
		rolling_block_number = rolling_block_number + 1;
		rolling_timestamp = rolling_timestamp + 10;

		if let Err(_) = client.import_block(create_test_block(&header)) {
			panic!("error importing block which is valid by definition");
		}

	}
	client.flush_queue();
	client.import_verified_blocks(&IoChannel::disconnected());
	client
}