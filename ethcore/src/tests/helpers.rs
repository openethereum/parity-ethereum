// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use account_provider::AccountProvider;
use bigint::hash::H256;
use bigint::prelude::U256;
use block::{OpenBlock, Drain};
use blockchain::{BlockChain, Config as BlockChainConfig};
use bytes::Bytes;
use client::{BlockChainClient, Client, ClientConfig};
use ethereum::ethash::EthashParams;
use ethkey::KeyPair;
use evm::Factory as EvmFactory;
use factory::Factories;
use hash::keccak;
use header::Header;
use io::*;
use machine::EthashExtensions;
use miner::Miner;
use rlp::{self, RlpStream};
use spec::*;
use state_db::StateDB;
use state::*;
use std::sync::Arc;
use transaction::{Action, Transaction, SignedTransaction};
use util::*;
use views::BlockView;

// TODO: move everything over to get_null_spec.
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

fn create_unverifiable_block_header(order: u32, parent_hash: H256) -> Header {
	let mut header = Header::new();
	header.set_gas_limit(0.into());
	header.set_difficulty((order * 100).into());
	header.set_timestamp((order * 10) as u64);
	header.set_number(order as u64);
	header.set_parent_hash(parent_hash);
	header.set_state_root(H256::zero());

	header
}

fn create_unverifiable_block_with_extra(order: u32, parent_hash: H256, extra: Option<Bytes>) -> Bytes {
	let mut header = create_unverifiable_block_header(order, parent_hash);
	header.set_extra_data(match extra {
		Some(extra_data) => extra_data,
		None => {
			let base = (order & 0x000000ff) as u8;
			let generated: Vec<u8> = vec![base + 1, base + 2, base + 3];
			generated
		}
	});
	create_test_block(&header)
}

fn create_unverifiable_block(order: u32, parent_hash: H256) -> Bytes {
	create_test_block(&create_unverifiable_block_header(order, parent_hash))
}

pub fn create_test_block_with_data(header: &Header, transactions: &[SignedTransaction], uncles: &[Header]) -> Bytes {
	let mut rlp = RlpStream::new_list(3);
	rlp.append(header);
	rlp.begin_list(transactions.len());
	for t in transactions {
		rlp.append_raw(&rlp::encode(t).into_vec(), 1);
	}
	rlp.append_list(&uncles);
	rlp.out()
}

pub fn generate_dummy_client(block_number: u32) -> Arc<Client> {
	generate_dummy_client_with_spec_and_data(Spec::new_test, block_number, 0, &[])
}

pub fn generate_dummy_client_with_data(block_number: u32, txs_per_block: usize, tx_gas_prices: &[U256]) -> Arc<Client> {
	generate_dummy_client_with_spec_and_data(Spec::new_null, block_number, txs_per_block, tx_gas_prices)
}


pub fn generate_dummy_client_with_spec_and_data<F>(get_test_spec: F, block_number: u32, txs_per_block: usize, tx_gas_prices: &[U256]) -> Arc<Client> where F: Fn()->Spec {
	generate_dummy_client_with_spec_accounts_and_data(get_test_spec, None, block_number, txs_per_block, tx_gas_prices)
}

pub fn generate_dummy_client_with_spec_and_accounts<F>(get_test_spec: F, accounts: Option<Arc<AccountProvider>>) -> Arc<Client> where F: Fn()->Spec {
	generate_dummy_client_with_spec_accounts_and_data(get_test_spec, accounts, 0, 0, &[])
}

pub fn generate_dummy_client_with_spec_accounts_and_data<F>(get_test_spec: F, accounts: Option<Arc<AccountProvider>>, block_number: u32, txs_per_block: usize, tx_gas_prices: &[U256]) -> Arc<Client> where F: Fn()->Spec {
	let test_spec = get_test_spec();
	let client_db = new_db();

	let client = Client::new(
		ClientConfig::default(),
		&test_spec,
		client_db,
		Arc::new(Miner::with_spec_and_accounts(&test_spec, accounts)),
		IoChannel::disconnected(),
	).unwrap();
	let test_engine = &*test_spec.engine;

	let mut db = test_spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
	let genesis_header = test_spec.genesis_header();

	let mut rolling_timestamp = 40;
	let mut last_hashes = vec![];
	let mut last_header = genesis_header.clone();

	let kp = KeyPair::from_secret_slice(&keccak("")).unwrap();
	let author = kp.address();

	let mut n = 0;
	for _ in 0..block_number {
		last_hashes.push(last_header.hash());

		// forge block.
		let mut b = OpenBlock::new(
			test_engine,
			Default::default(),
			false,
			db,
			&last_header,
			Arc::new(last_hashes.clone()),
			author.clone(),
			(3141562.into(), 31415620.into()),
			vec![],
			false,
		).unwrap();
		b.set_difficulty(U256::from(0x20000));
		rolling_timestamp += 10;
		b.set_timestamp(rolling_timestamp);

		// first block we don't have any balance, so can't send any transactions.
		for _ in 0..txs_per_block {
			b.push_transaction(Transaction {
				nonce: n.into(),
				gas_price: tx_gas_prices[n % tx_gas_prices.len()],
				gas: 100000.into(),
				action: Action::Create,
				data: vec![],
				value: U256::zero(),
			}.sign(kp.secret(), Some(test_spec.chain_id())), None).unwrap();
			n += 1;
		}

		let b = b.close_and_lock().seal(test_engine, vec![]).unwrap();

		if let Err(e) = client.import_block(b.rlp_bytes()) {
			panic!("error importing block which is valid by definition: {:?}", e);
		}

		last_header = BlockView::new(&b.rlp_bytes()).header();
		db = b.drain();
	}
	client.flush_queue();
	client.import_verified_blocks();
	client
}

pub fn push_blocks_to_client(client: &Arc<Client>, timestamp_salt: u64, starting_number: usize, block_number: usize) {
	let test_spec = get_test_spec();
	let state_root = test_spec.genesis_header().state_root().clone();
	let genesis_gas = test_spec.genesis_header().gas_limit().clone();

	let mut rolling_hash = client.chain_info().best_block_hash;
	let mut rolling_block_number = starting_number as u64;
	let mut rolling_timestamp = timestamp_salt + starting_number as u64 * 10;

	for _ in 0..block_number {
		let mut header = Header::new();

		header.set_gas_limit(genesis_gas);
		header.set_difficulty(U256::from(0x20000));
		header.set_timestamp(rolling_timestamp);
		header.set_number(rolling_block_number);
		header.set_parent_hash(rolling_hash);
		header.set_state_root(state_root);

		rolling_hash = header.hash();
		rolling_block_number = rolling_block_number + 1;
		rolling_timestamp = rolling_timestamp + 10;

		if let Err(e) = client.import_block(create_test_block(&header)) {
			panic!("error importing block which is valid by definition: {:?}", e);
		}
	}
}

pub fn get_test_client_with_blocks(blocks: Vec<Bytes>) -> Arc<Client> {
	let test_spec = get_test_spec();
	let client_db = new_db();

	let client = Client::new(
		ClientConfig::default(),
		&test_spec,
		client_db,
		Arc::new(Miner::with_spec(&test_spec)),
		IoChannel::disconnected(),
	).unwrap();

	for block in blocks {
		if let Err(e) = client.import_block(block) {
			panic!("error importing block which is well-formed: {:?}", e);
		}
	}
	client.flush_queue();
	client.import_verified_blocks();
	client
}

fn new_db() -> Arc<::kvdb::KeyValueDB> {
	Arc::new(::kvdb_memorydb::in_memory(::db::NUM_COLUMNS.unwrap_or(0)))
}

pub fn generate_dummy_blockchain(block_number: u32) -> BlockChain {
	let db = new_db();
	let bc = BlockChain::new(BlockChainConfig::default(), &create_unverifiable_block(0, H256::zero()), db.clone());

	let mut batch = db.transaction();
	for block_order in 1..block_number {
		bc.insert_block(&mut batch, &create_unverifiable_block(block_order, bc.best_block_hash()), vec![]);
		bc.commit();
	}
	db.write(batch).unwrap();
	bc
}

pub fn generate_dummy_blockchain_with_extra(block_number: u32) -> BlockChain {
	let db = new_db();
	let bc = BlockChain::new(BlockChainConfig::default(), &create_unverifiable_block(0, H256::zero()), db.clone());


	let mut batch = db.transaction();
	for block_order in 1..block_number {
		bc.insert_block(&mut batch, &create_unverifiable_block_with_extra(block_order, bc.best_block_hash(), None), vec![]);
		bc.commit();
	}
	db.write(batch).unwrap();
	bc
}

pub fn generate_dummy_empty_blockchain() -> BlockChain {
	let db = new_db();
	let bc = BlockChain::new(BlockChainConfig::default(), &create_unverifiable_block(0, H256::zero()), db.clone());
	bc
}

pub fn get_temp_state() -> State<::state_db::StateDB> {
	let journal_db = get_temp_state_db();
	State::new(journal_db, U256::from(0), Default::default())
}

pub fn get_temp_state_with_factory(factory: EvmFactory) -> State<::state_db::StateDB> {
	let journal_db = get_temp_state_db();
	let mut factories = Factories::default();
	factories.vm = factory;
	State::new(journal_db, U256::from(0), factories)
}

pub fn get_temp_state_db() -> StateDB {
	let db = new_db();
	let journal_db = journaldb::new(db, journaldb::Algorithm::EarlyMerge, ::db::COL_STATE);
	StateDB::new(journal_db, 5 * 1024 * 1024)
}

pub fn get_good_dummy_block_seq(count: usize) -> Vec<Bytes> {
	let test_spec = get_test_spec();
	get_good_dummy_block_fork_seq(1, count, &test_spec.genesis_header().hash())
}

pub fn get_good_dummy_block_fork_seq(start_number: usize, count: usize, parent_hash: &H256) -> Vec<Bytes> {
	let test_spec = get_test_spec();
	let genesis_gas = test_spec.genesis_header().gas_limit().clone();
	let mut rolling_timestamp = start_number as u64 * 10;
	let mut parent = *parent_hash;
	let mut r = Vec::new();
	for i in start_number .. start_number + count + 1 {
		let mut block_header = Header::new();
		block_header.set_gas_limit(genesis_gas);
		block_header.set_difficulty(U256::from(i) * U256([0, 1, 0, 0]));
		block_header.set_timestamp(rolling_timestamp);
		block_header.set_number(i as u64);
		block_header.set_parent_hash(parent);
		block_header.set_state_root(test_spec.genesis_header().state_root().clone());

		parent = block_header.hash();
		rolling_timestamp = rolling_timestamp + 10;

		r.push(create_test_block(&block_header));
	}
	r
}

pub fn get_good_dummy_block_hash() -> (H256, Bytes) {
	let mut block_header = Header::new();
	let test_spec = get_test_spec();
	let genesis_gas = test_spec.genesis_header().gas_limit().clone();
	block_header.set_gas_limit(genesis_gas);
	block_header.set_difficulty(U256::from(0x20000));
	block_header.set_timestamp(40);
	block_header.set_number(1);
	block_header.set_parent_hash(test_spec.genesis_header().hash());
	block_header.set_state_root(test_spec.genesis_header().state_root().clone());

	(block_header.hash(), create_test_block(&block_header))
}

pub fn get_good_dummy_block() -> Bytes {
	let (_, bytes) = get_good_dummy_block_hash();
	bytes
}

pub fn get_bad_state_dummy_block() -> Bytes {
	let mut block_header = Header::new();
	let test_spec = get_test_spec();
	let genesis_gas = test_spec.genesis_header().gas_limit().clone();

	block_header.set_gas_limit(genesis_gas);
	block_header.set_difficulty(U256::from(0x20000));
	block_header.set_timestamp(40);
	block_header.set_number(1);
	block_header.set_parent_hash(test_spec.genesis_header().hash());
	block_header.set_state_root(0xbad.into());

	create_test_block(&block_header)
}

pub fn get_default_ethash_extensions() -> EthashExtensions {
	EthashExtensions {
		homestead_transition: 1150000,
		eip150_transition: u64::max_value(),
		eip160_transition: u64::max_value(),
		eip161abc_transition: u64::max_value(),
		eip161d_transition: u64::max_value(),
		dao_hardfork_transition: u64::max_value(),
		dao_hardfork_beneficiary: "0000000000000000000000000000000000000001".into(),
		dao_hardfork_accounts: Vec::new(),
	}
}

pub fn get_default_ethash_params() -> EthashParams {
	EthashParams {
		minimum_difficulty: U256::from(131072),
		difficulty_bound_divisor: U256::from(2048),
		difficulty_increment_divisor: 10,
		metropolis_difficulty_increment_divisor: 9,
		homestead_transition: 1150000,
		duration_limit: 13,
		block_reward: 0.into(),
		difficulty_hardfork_transition: u64::max_value(),
		difficulty_hardfork_bound_divisor: U256::from(0),
		bomb_defuse_transition: u64::max_value(),
		eip100b_transition: u64::max_value(),
		ecip1010_pause_transition: u64::max_value(),
		ecip1010_continue_transition: u64::max_value(),
		ecip1017_era_rounds: u64::max_value(),
		mcip3_transition: u64::max_value(),
		mcip3_miner_reward: 0.into(),
		mcip3_ubi_reward: 0.into(),
		mcip3_ubi_contract: "0000000000000000000000000000000000000001".into(),
		mcip3_dev_reward: 0.into(),
		mcip3_dev_contract: "0000000000000000000000000000000000000001".into(),
		eip649_transition: u64::max_value(),
		eip649_delay: 3_000_000,
		eip649_reward: None,
	}
}
