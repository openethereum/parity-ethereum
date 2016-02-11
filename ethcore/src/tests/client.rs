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

use client::{BlockChainClient,Client};
use tests::helpers::*;
use common::*;

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
fn returns_block_body() {
	let dummy_block = get_good_dummy_block();
	let client_result = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let client = client_result.reference();
	let block = BlockView::new(&dummy_block);
	let body = client.block_body(&block.header().hash()).unwrap();
	let body = Rlp::new(&body);
	assert_eq!(body.item_count(), 2);
	assert_eq!(body.at(0).as_raw()[..], block.rlp().at(1).as_raw()[..]);
	assert_eq!(body.at(1).as_raw()[..], block.rlp().at(2).as_raw()[..]);
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
