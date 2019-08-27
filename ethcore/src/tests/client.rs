// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::str::FromStr;
use std::sync::Arc;

use ethereum_types::{U256, Address};
use ethkey::KeyPair;
use hash::keccak;
use io::IoChannel;
use tempdir::TempDir;
use types::{
	ids::BlockId,
	transaction::{PendingTransaction, Transaction, Action, Condition},
	filter::Filter,
	verification::Unverified,
	view,
	views::BlockView,
};

use client::{Client, ClientConfig, PrepareOpenBlock, ImportSealedBlock};
use client_traits::{BlockInfo, BlockChainClient, BlockChainReset, ChainInfo, ImportBlock};
use spec;
use machine::executive::{Executive, TransactOptions};
use miner::{Miner, PendingOrdering, MinerService};
use account_state::{State, CleanupMode, backend};
use test_helpers::{
	self,
	generate_dummy_client, push_blocks_to_client, get_test_client_with_blocks, get_good_dummy_block_seq,
	generate_dummy_client_with_data, get_good_dummy_block, get_bad_state_dummy_block
};

#[test]
fn imports_from_empty() {
	let db = test_helpers::new_db();
	let spec = spec::new_test();

	let client = Client::new(
		ClientConfig::default(),
		&spec,
		db,
		Arc::new(Miner::new_for_tests(&spec, None)),
		IoChannel::disconnected(),
	).unwrap();
	client.import_verified_blocks();
	client.flush_queue();
}

#[test]
fn should_return_registrar() {
	let db = test_helpers::new_db();
	let tempdir = TempDir::new("").unwrap();
	let spec = spec::new_morden(&tempdir.path().to_owned());

	let client = Client::new(
		ClientConfig::default(),
		&spec,
		db,
		Arc::new(Miner::new_for_tests(&spec, None)),
		IoChannel::disconnected(),
	).unwrap();
	let address = client.registrar_address();
	assert_eq!(address, Some("52dff57a8a1532e6afb3dc07e2af58bb9eb05b3d".parse().unwrap()));
}

#[test]
fn returns_state_root_basic() {
	let client = generate_dummy_client(6);
	let test_spec = spec::new_test();
	let genesis_header = test_spec.genesis_header();

	assert!(client.state_data(genesis_header.state_root()).is_some());
}

#[test]
fn imports_good_block() {
	let db = test_helpers::new_db();
	let spec = spec::new_test();

	let client = Client::new(
		ClientConfig::default(),
		&spec,
		db,
		Arc::new(Miner::new_for_tests(&spec, None)),
		IoChannel::disconnected(),
	).unwrap();
	let good_block = get_good_dummy_block();
	if client.import_block(Unverified::from_rlp(good_block).unwrap()).is_err() {
		panic!("error importing block being good by definition");
	}
	client.flush_queue();
	client.import_verified_blocks();

	let block = client.block_header(BlockId::Number(1)).unwrap();
	assert!(!block.into_inner().is_empty());
}

#[test]
fn query_none_block() {
	let db = test_helpers::new_db();
	let spec = spec::new_test();

	let client = Client::new(
		ClientConfig::default(),
		&spec,
		db,
		Arc::new(Miner::new_for_tests(&spec, None)),
		IoChannel::disconnected(),
	).unwrap();
	let non_existant = client.block_header(BlockId::Number(188));
	assert!(non_existant.is_none());
}

#[test]
fn query_bad_block() {
	let client = get_test_client_with_blocks(vec![get_bad_state_dummy_block()]);
	let bad_block: Option<_> = client.block_header(BlockId::Number(1));

	assert!(bad_block.is_none());
}

#[test]
fn returns_chain_info() {
	let dummy_block = get_good_dummy_block();
	let client = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let block = view!(BlockView, &dummy_block);
	let info = client.chain_info();
	assert_eq!(info.best_block_hash, block.header().hash());
}

#[test]
fn returns_logs() {
	let dummy_block = get_good_dummy_block();
	let client = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let logs = client.logs(Filter {
		from_block: BlockId::Earliest,
		to_block: BlockId::Latest,
		address: None,
		topics: vec![],
		limit: None,
	}).unwrap();
	assert_eq!(logs.len(), 0);
}

#[test]
fn returns_logs_with_limit() {
	let dummy_block = get_good_dummy_block();
	let client = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let logs = client.logs(Filter {
		from_block: BlockId::Earliest,
		to_block: BlockId::Latest,
		address: None,
		topics: vec![],
		limit: None,
	}).unwrap();
	assert_eq!(logs.len(), 0);
}

#[test]
fn returns_block_body() {
	let dummy_block = get_good_dummy_block();
	let client = get_test_client_with_blocks(vec![dummy_block.clone()]);
	let block = view!(BlockView, &dummy_block);
	let body = client.block_body(BlockId::Hash(block.header().hash())).unwrap();
	let body = body.rlp();
	assert_eq!(body.item_count().unwrap(), 2);
	assert_eq!(body.at(0).unwrap().as_raw()[..], block.rlp().at(1).as_raw()[..]);
	assert_eq!(body.at(1).unwrap().as_raw()[..], block.rlp().at(2).as_raw()[..]);
}

#[test]
fn imports_block_sequence() {
	let client = generate_dummy_client(6);
	let block = client.block_header(BlockId::Number(5)).unwrap();

	assert!(!block.into_inner().is_empty());
}

#[test]
fn can_collect_garbage() {
	let client = generate_dummy_client(100);
	client.tick(true);
	assert!(client.blockchain_cache_info().blocks < 100 * 1024);
}

#[test]
fn can_generate_gas_price_median() {
	let client = generate_dummy_client_with_data(3, 1, slice_into![1, 2, 3]);
	assert_eq!(Some(&U256::from(2)), client.gas_price_corpus(3).median());

	let client = generate_dummy_client_with_data(4, 1, slice_into![1, 4, 3, 2]);
	assert_eq!(Some(&U256::from(3)), client.gas_price_corpus(3).median());
}

#[test]
fn can_generate_gas_price_histogram() {
	let client = generate_dummy_client_with_data(20, 1, slice_into![6354,8593,6065,4842,7845,7002,689,4958,4250,6098,5804,4320,643,8895,2296,8589,7145,2000,2512,1408]);

	let hist = client.gas_price_corpus(20).histogram(5).unwrap();
	let correct_hist = stats::Histogram { bucket_bounds: vec_into![643, 2294, 3945, 5596, 7247, 8898], counts: vec![4,2,4,6,4] };
	assert_eq!(hist, correct_hist);
}

#[test]
fn empty_gas_price_histogram() {
	let client = generate_dummy_client_with_data(20, 0, slice_into![]);

	assert!(client.gas_price_corpus(20).histogram(5).is_none());
}

#[test]
fn corpus_is_sorted() {
	let client = generate_dummy_client_with_data(2, 1, slice_into![U256::from_str("11426908979").unwrap(), U256::from_str("50426908979").unwrap()]);
	let corpus = client.gas_price_corpus(20);
	assert!(corpus[0] < corpus[1]);
}

#[test]
fn can_handle_long_fork() {
	let client = generate_dummy_client(1200);
	for _ in 0..20 {
		client.import_verified_blocks();
	}
	assert_eq!(1200, client.chain_info().best_block_number);

	push_blocks_to_client(&client, 45, 1201, 800);
	push_blocks_to_client(&client, 49, 1201, 800);
	push_blocks_to_client(&client, 53, 1201, 600);

	for _ in 0..400 {
		client.import_verified_blocks();
	}
	assert_eq!(2000, client.chain_info().best_block_number);
}

#[test]
fn can_mine() {
	let dummy_blocks = get_good_dummy_block_seq(2);
	let client = get_test_client_with_blocks(vec![dummy_blocks[0].clone()]);

	let b = client.prepare_open_block(Address::zero(), (3141562.into(), 31415620.into()), vec![]).unwrap().close().unwrap();

	assert_eq!(*b.header.parent_hash(), view!(BlockView, &dummy_blocks[0]).header_view().hash());
}

#[test]
fn change_history_size() {
	let db = test_helpers::new_db();
	let test_spec = spec::new_null();
	let mut config = ClientConfig::default();

	config.history = 2;
	let address = Address::random();
	{
		let client = Client::new(
			ClientConfig::default(),
			&test_spec,
			db.clone(),
			Arc::new(Miner::new_for_tests(&test_spec, None)),
			IoChannel::disconnected()
		).unwrap();

		for _ in 0..20 {
			let mut b = client.prepare_open_block(Address::zero(), (3141562.into(), 31415620.into()), vec![]).unwrap();
			b.block_mut().state_mut().add_balance(&address, &5.into(), CleanupMode::NoEmpty).unwrap();
			b.block_mut().state_mut().commit().unwrap();
			let b = b.close_and_lock().unwrap().seal(&*test_spec.engine, vec![]).unwrap();
			client.import_sealed_block(b).unwrap(); // account change is in the journal overlay
		}
	}
	let mut config = ClientConfig::default();
	config.history = 10;
	let client = Client::new(
		config,
		&test_spec,
		db,
		Arc::new(Miner::new_for_tests(&test_spec, None)),
		IoChannel::disconnected(),
	).unwrap();
	assert_eq!(client.state().balance(&address).unwrap(), 100.into());
}

#[test]
fn does_not_propagate_delayed_transactions() {
	let key = KeyPair::from_secret(keccak("test").into()).unwrap();
	let secret = key.secret();
	let tx0 = PendingTransaction::new(Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::zero()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(secret, None), Some(Condition::Number(2)));
	let tx1 = PendingTransaction::new(Transaction {
		nonce: 1.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::zero()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(secret, None), None);
	let client = generate_dummy_client(1);

	client.miner().import_own_transaction(&*client, tx0).unwrap();
	client.miner().import_own_transaction(&*client, tx1).unwrap();
	assert_eq!(0, client.transactions_to_propagate().len());
	assert_eq!(0, client.miner().ready_transactions(&*client, 10, PendingOrdering::Priority).len());
	push_blocks_to_client(&client, 53, 2, 2);
	client.flush_queue();
	assert_eq!(2, client.transactions_to_propagate().len());
	assert_eq!(2, client.miner().ready_transactions(&*client, 10, PendingOrdering::Priority).len());
}

#[test]
fn transaction_proof() {
	use client_traits::ProvingBlockChainClient;

	let client = generate_dummy_client(0);
	let address = Address::random();
	let test_spec = spec::new_test();
	for _ in 0..20 {
		let mut b = client.prepare_open_block(Address::zero(), (3141562.into(), 31415620.into()), vec![]).unwrap();
		b.block_mut().state_mut().add_balance(&address, &5.into(), CleanupMode::NoEmpty).unwrap();
		b.block_mut().state_mut().commit().unwrap();
		let b = b.close_and_lock().unwrap().seal(&*test_spec.engine, vec![]).unwrap();
		client.import_sealed_block(b).unwrap(); // account change is in the journal overlay
	}

	let transaction = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::zero()),
		value: 5.into(),
		data: Vec::new(),
	}.fake_sign(address);

	let proof = client.prove_transaction(transaction.clone(), BlockId::Latest).unwrap().1;
	let backend = backend::ProofCheck::new(&proof);

	let mut factories = ::trie_vm_factories::Factories::default();
	factories.accountdb = ::account_db::Factory::Plain; // raw state values, no mangled keys.
	let root = *client.best_block_header().state_root();

	let machine = test_spec.engine.machine();
	let env_info = client.latest_env_info();
	let schedule = machine.schedule(env_info.number);
	let mut state = State::from_existing(backend, root, 0.into(), factories.clone()).unwrap();
	Executive::new(&mut state, &env_info, &machine, &schedule)
		.transact(&transaction, TransactOptions::with_no_tracing().dont_check_nonce()).unwrap();

	assert_eq!(state.balance(&Address::zero()).unwrap(), 5.into());
	assert_eq!(state.balance(&address).unwrap(), 95.into());
}

#[test]
fn reset_blockchain() {
	let client = get_test_client_with_blocks(get_good_dummy_block_seq(19));
	// 19 + genesis block
	assert!(client.block_header(BlockId::Number(20)).is_some());
	assert_eq!(client.block_header(BlockId::Number(20)).unwrap().hash(), client.best_block_header().hash());

	assert!(client.reset(5).is_ok());

	client.chain().clear_cache();

	assert!(client.block_header(BlockId::Number(20)).is_none());
	assert!(client.block_header(BlockId::Number(19)).is_none());
	assert!(client.block_header(BlockId::Number(18)).is_none());
	assert!(client.block_header(BlockId::Number(17)).is_none());
	assert!(client.block_header(BlockId::Number(16)).is_none());

	assert!(client.block_header(BlockId::Number(15)).is_some());
}
