use client::{BlockChainClient,Client};
use std::env;
use super::test_common::*;
use std::path::PathBuf;
use spec::*;
use std::fs::{create_dir_all};


const FIXED_TEMP_DIR_NAME: &'static str = "parity-temp";


pub fn get_tests_temp_dir() -> PathBuf {
	let mut dir = env::temp_dir();
	dir.push(FIXED_TEMP_DIR_NAME);
	if let Err(_) = create_dir_all(&dir) {
		panic!("failed to create test dir!");
	}
	dir
}

pub fn get_random_path() -> PathBuf {
	let mut dir = get_tests_temp_dir();
	dir.push(H32::random().hex());
	dir
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

pub fn generate_dummy_client(block_number: usize) -> Arc<Client> {
	let client = Client::new(get_test_spec(), &get_random_path(), IoChannel::disconnected()).unwrap();

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