use client::{BlockChainClient,Client};
use std::env;
use super::test_common::*;
use std::path::PathBuf;
use spec::*;
use std::fs::{create_dir_all};

#[cfg(test)]
const FIXED_TEMP_DIR_NAME: &'static str = "parity-temp";


#[cfg(test)]
pub fn get_tests_temp_dir() -> PathBuf {
	let mut dir = env::temp_dir();
	dir.push(FIXED_TEMP_DIR_NAME);
	if let Err(_) = create_dir_all(&dir) {
		panic!("failed to create test dir!");
	}
	dir
}

#[cfg(test)]
pub fn get_random_path() -> PathBuf {
	let mut dir = get_tests_temp_dir();
	dir.push(H32::random().hex());
	dir
}


#[cfg(test)]
pub fn get_test_spec() -> Spec {
	Spec::new_test()
}

#[cfg(test)]
pub fn generate_dummy_client(block_number: usize) {
	let client = Client::new(get_test_spec(), &get_random_path(), IoChannel::disconnected()).unwrap();

	let mut rolling_hash = test_engine.spec().genesis_header().hash();
	let mut rolling_state = test_engine.spec().genesis_header().state_root;
	let mut rolling_block_number = 1;

	for _ in 0..block_number {
		let mut header = Header::new();

		header.gas_limit = decode(test_engine.spec().engine_params.get("minGasLimit").unwrap());
		header.difficulty = decode(test_engine.spec().engine_params.get("minimumDifficulty").unwrap());
		header.timestamp = 40;
		header.number = rolling_block_number;
		header.parent_hash = test_engine.spec().genesis_header().hash();
		header.state_root = test_engine.spec().genesis_header().state_root;
	}

}