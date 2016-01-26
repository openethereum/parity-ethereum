use client::{BlockChainClient,Client};
use std::env;
use super::test_common::*;
use std::path::PathBuf;
use spec::*;

#[cfg(test)]
fn get_random_temp_dir() -> PathBuf {
	let mut dir = env::temp_dir();
	dir.push(H32::random().hex());
	dir
}

#[cfg(test)]
fn get_test_spec() -> Spec {
	Spec::new_test()
}

#[cfg(test)]
fn get_good_dummy_block() -> Bytes {
	let mut block_header = Header::new();
	let test_spec = get_test_spec();
	let test_engine = test_spec.to_engine().unwrap();
	block_header.gas_limit = decode(test_engine.spec().engine_params.get("minGasLimit").unwrap());
	block_header.difficulty = decode(test_engine.spec().engine_params.get("minimumDifficulty").unwrap());
	block_header.timestamp = 40;
	block_header.number = 1;
	block_header.parent_hash = test_engine.spec().genesis_header().hash();
	block_header.state_root = test_engine.spec().genesis_header().state_root;

	create_test_block(&block_header)
}

#[cfg(test)]
fn get_bad_state_dummy_block() -> Bytes {
	let mut block_header = Header::new();
	let test_spec = get_test_spec();
	let test_engine = test_spec.to_engine().unwrap();
	block_header.gas_limit = decode(test_engine.spec().engine_params.get("minGasLimit").unwrap());
	block_header.difficulty = decode(test_engine.spec().engine_params.get("minimumDifficulty").unwrap());
	block_header.timestamp = 40;
	block_header.number = 1;
	block_header.parent_hash = test_engine.spec().genesis_header().hash();
	block_header.state_root = x!(0xbad);

	create_test_block(&block_header)
}

#[cfg(test)]
fn create_test_block(header: &Header) -> Bytes {
	let mut rlp = RlpStream::new_list(3);
	rlp.append(header);
	rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
	rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
	rlp.out()
}

#[cfg(test)]
fn get_test_client_with_blocks(blocks: Vec<Bytes>) -> Arc<Client> {
	let client = Client::new(get_test_spec(), &get_random_temp_dir(), IoChannel::disconnected()).unwrap();

	for block in &blocks {
		if let Err(_) = client.import_block(block.clone()) {
			panic!("panic importing block which is well-formed");
		}
	}

	client.flush_queue();

	client.import_verified_blocks(&IoChannel::disconnected());

	client
}


#[test]
fn created() {
	let client_result = Client::new(get_test_spec(), &get_random_temp_dir(), IoChannel::disconnected());
	assert!(client_result.is_ok());
}

#[test]
fn imports_from_empty() {
	let client = Client::new(get_test_spec(), &get_random_temp_dir(), IoChannel::disconnected()).unwrap();
	client.import_verified_blocks(&IoChannel::disconnected());
	client.flush_queue();
}

#[test]
fn imports_good_block() {
	let client = Client::new(get_test_spec(), &get_random_temp_dir(), IoChannel::disconnected()).unwrap();

	let good_block = get_good_dummy_block();
	if let Err(_) = client.import_block(good_block) {
		panic!("error importing block being good by definition");
	}

	client.flush_queue();

	client.import_verified_blocks(&IoChannel::disconnected());

	let block = client.block_header_at(1).unwrap();

	assert!(!block.is_empty());
}

#[test]
fn query_none_block() {
	let client = Client::new(get_test_spec(), &get_random_temp_dir(), IoChannel::disconnected()).unwrap();

    let non_existant = client.block_header_at(188);

	assert!(non_existant.is_none());
}

#[test]
fn query_bad_block() {
	let client = get_test_client_with_blocks(vec![get_bad_state_dummy_block()]);
	let bad_block:Option<Bytes> = client.block_header_at(1);

	assert!(bad_block.is_none());
}

#[test]
fn returns_chain_info() {
	let dummy_block = get_good_dummy_block();
	let client = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let block = BlockView::new(&dummy_block);
	let info = client.chain_info();
	assert_eq!(info.best_block_hash, block.header().hash());
}