use client::{BlockChainClient,Client};
use super::test_common::*;
use tests::helpers::*;

#[test]
fn created() {
	let dir = RandomTempPath::new();
	let client_result = Client::new(get_test_spec(), dir.as_path(), IoChannel::disconnected());
	assert!(client_result.is_ok());
}

#[test]
fn imports_from_empty() {
	let dir = RandomTempPath::new();
	let client = Client::new(get_test_spec(), dir.as_path(), IoChannel::disconnected()).unwrap();
	client.import_verified_blocks(&IoChannel::disconnected());
	client.flush_queue();
}

#[test]
fn imports_good_block() {
	let dir = RandomTempPath::new();
	let client = Client::new(get_test_spec(), dir.as_path(), IoChannel::disconnected()).unwrap();
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
	let dir = RandomTempPath::new();
	let client = Client::new(get_test_spec(), dir.as_path(), IoChannel::disconnected()).unwrap();

    let non_existant = client.block_header_at(188);
	assert!(non_existant.is_none());
}

#[test]
fn query_bad_block() {
	let client_result = get_test_client_with_blocks(vec![get_bad_state_dummy_block()]);
	let client = client_result.reference();
	let bad_block:Option<Bytes> = client.block_header_at(1);

	assert!(bad_block.is_none());
}

#[test]
fn returns_chain_info() {
	let dummy_block = get_good_dummy_block();
	let client_result = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let client = client_result.reference();
	let block = BlockView::new(&dummy_block);
	let info = client.chain_info();
	assert_eq!(info.best_block_hash, block.header().hash());
}

#[test]
fn imports_block_sequence() {
	let client_result = generate_dummy_client(6);
	let client = client_result.reference();
	let block = client.block_header_at(5).unwrap();

	assert!(!block.is_empty());
}

#[test]
fn can_collect_garbage() {
	let client_result = generate_dummy_client(100);
	let client = client_result.reference();
	client.tick();
	assert!(client.cache_info().blocks < 100 * 1024);
}
