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

use client;
use client::{BlockChainClient, Client, ClientConfig, BlockID};
use block::IsBlock;
use tests::helpers::*;
use common::*;
use devtools::*;
use std::sync::atomic::{AtomicBool, Ordering as SyncOrdering};
use ethcore_db;

#[test]
fn imports_from_empty() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let client = Client::new(ClientConfig::default(), get_test_spec(), dir.as_path(), IoChannel::disconnected());

		client.import_verified_blocks(&IoChannel::disconnected());
		client.flush_queue();

		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn returns_state_root_basic() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let client = generate_dummy_client_in(&dir, 6);
		let test_spec = get_test_spec();
		let state_root = test_spec.genesis_header().state_root;

		assert!(client.state_data(&state_root).is_some());
		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn imports_good_block() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let client = Client::new(ClientConfig::default(), get_test_spec(), dir.as_path(), IoChannel::disconnected());
		let good_block = get_good_dummy_block();
		if let Err(_) = client.import_block(good_block) {
			panic!("error importing block being good by definition");
		}
		client.flush_queue();
		client.import_verified_blocks(&IoChannel::disconnected());

		let block = client.block_header(BlockID::Number(1)).unwrap();
		assert!(!block.is_empty());
		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn query_none_block() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let client = Client::new(ClientConfig::default(), get_test_spec(), dir.as_path(), IoChannel::disconnected());
		let non_existant = client.block_header(BlockID::Number(188));
		assert!(non_existant.is_none());
		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn query_bad_block() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let client = get_test_client_with_blocks_in(&dir, vec![get_bad_state_dummy_block()]);
		let bad_block:Option<Bytes> = client.block_header(BlockID::Number(1));

		assert!(bad_block.is_none());
		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn returns_chain_info() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let dummy_block = get_good_dummy_block();
		let client = get_test_client_with_blocks_in(&dir, vec![dummy_block.clone()]);
		let block = BlockView::new(&dummy_block);
		let info = client.chain_info();
		assert_eq!(info.best_block_hash, block.header().hash());

		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn returns_block_body() {

	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let dummy_block = get_good_dummy_block();
		let client = get_test_client_with_blocks_in(&dir, vec![dummy_block.clone()]);
		let block = BlockView::new(&dummy_block);
		let body = client.block_body(BlockID::Hash(block.header().hash())).unwrap();
		let body = Rlp::new(&body);
		assert_eq!(body.item_count(), 2);
		assert_eq!(body.at(0).as_raw()[..], block.rlp().at(1).as_raw()[..]);
		assert_eq!(body.at(1).as_raw()[..], block.rlp().at(2).as_raw()[..]);

		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn imports_block_sequence() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let client = generate_dummy_client_in(&dir, 6);
		let block = client.block_header(BlockID::Number(5)).unwrap();

		assert!(!block.is_empty());

		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn cl_can_collect_garbage() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let client = generate_dummy_client_in(&dir, 100);
		client.tick();
		assert!(client.blockchain_cache_info().blocks < 100 * 1024);

		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn can_handle_long_fork() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());


		let client = generate_dummy_client_in(&dir, 1200);

		for _ in 0..10 {
			client.import_verified_blocks(&IoChannel::disconnected());
		}
		assert_eq!(1200, client.chain_info().best_block_number);

		push_blocks_to_client(&client, 45, 1201, 800);
		push_blocks_to_client(&client, 49, 1201, 800);
		push_blocks_to_client(&client, 53, 1201, 600);

		for _ in 0..20 {
			client.import_verified_blocks(&IoChannel::disconnected());
		}
		assert_eq!(2000, client.chain_info().best_block_number);

		stop.store(true, SyncOrdering::Relaxed);
	});
}

#[test]
fn can_mine() {
	let dir = RandomTempPath::new();
	let db_path = client::get_db_path(
		dir.as_path(),
		ClientConfig::default().pruning,
		get_test_spec().genesis_header().hash()).to_str().unwrap().to_owned();

	::crossbeam::scope(|scope| {
		let stop = Arc::new(AtomicBool::new(false));
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::extras_service_url(&db_path).unwrap());
		ethcore_db::run_worker(scope, stop.clone(), &ethcore_db::blocks_service_url(&db_path).unwrap());

		let dummy_blocks = get_good_dummy_block_seq(2);
		let client = get_test_client_with_blocks_in(&dir, vec![dummy_blocks[0].clone()]);

		let b = client.prepare_sealing(Address::default(), x!(31415926), vec![], vec![]).0.unwrap();

		assert_eq!(*b.block().header().parent_hash(), BlockView::new(&dummy_blocks[0]).header_view().sha3());
		assert!(client.try_seal(b.lock(), vec![]).is_ok());

		stop.store(true, SyncOrdering::Relaxed);
	});
}
