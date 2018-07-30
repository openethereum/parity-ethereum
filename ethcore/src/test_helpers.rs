// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Set of different helpers for client tests

use std::path::Path;
use std::sync::Arc;
use std::{fs, io};
use account_provider::AccountProvider;
use ethereum_types::{H256, U256, Address};
use block::{OpenBlock, Drain};
use blockchain::{BlockChain, BlockChainDB, BlockChainDBHandler, Config as BlockChainConfig, ExtrasInsert};
use bytes::Bytes;
use client::{Client, ClientConfig, ChainInfo, ImportBlock, ChainNotify, ChainMessageType, PrepareOpenBlock};
use ethkey::KeyPair;
use evm::Factory as EvmFactory;
use factory::Factories;
use hash::keccak;
use header::Header;
use io::*;
use miner::Miner;
use parking_lot::RwLock;
use rlp::{self, RlpStream};
use spec::Spec;
use state_db::StateDB;
use state::*;
use transaction::{Action, Transaction, SignedTransaction};
use views::BlockView;
use blooms_db;
use kvdb::KeyValueDB;
use kvdb_rocksdb;
use tempdir::TempDir;
use verification::queue::kind::blocks::Unverified;
use encoded;

/// Creates test block with corresponding header
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

/// Creates test block with corresponding header and data
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

/// Generates dummy client (not test client) with corresponding amount of blocks
pub fn generate_dummy_client(block_number: u32) -> Arc<Client> {
	generate_dummy_client_with_spec_and_data(Spec::new_test, block_number, 0, &[])
}

/// Generates dummy client (not test client) with corresponding amount of blocks and txs per every block
pub fn generate_dummy_client_with_data(block_number: u32, txs_per_block: usize, tx_gas_prices: &[U256]) -> Arc<Client> {
	generate_dummy_client_with_spec_and_data(Spec::new_null, block_number, txs_per_block, tx_gas_prices)
}

/// Generates dummy client (not test client) with corresponding amount of blocks, txs per block and spec
pub fn generate_dummy_client_with_spec_and_data<F>(test_spec: F, block_number: u32, txs_per_block: usize, tx_gas_prices: &[U256]) -> Arc<Client> where F: Fn()->Spec {
	generate_dummy_client_with_spec_accounts_and_data(test_spec, None, block_number, txs_per_block, tx_gas_prices)
}

/// Generates dummy client (not test client) with corresponding spec and accounts
pub fn generate_dummy_client_with_spec_and_accounts<F>(test_spec: F, accounts: Option<Arc<AccountProvider>>) -> Arc<Client> where F: Fn()->Spec {
	generate_dummy_client_with_spec_accounts_and_data(test_spec, accounts, 0, 0, &[])
}

/// Generates dummy client (not test client) with corresponding blocks, accounts and spec
pub fn generate_dummy_client_with_spec_accounts_and_data<F>(test_spec: F, accounts: Option<Arc<AccountProvider>>, block_number: u32, txs_per_block: usize, tx_gas_prices: &[U256]) -> Arc<Client> where F: Fn()->Spec {
	let test_spec = test_spec();
	let client_db = new_db();

	let client = Client::new(
		ClientConfig::default(),
		&test_spec,
		client_db,
		Arc::new(Miner::new_for_tests(&test_spec, accounts)),
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
			&mut Vec::new().into_iter(),
		).unwrap();
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

		let b = b.close_and_lock().unwrap().seal(test_engine, vec![]).unwrap();

		if let Err(e) = client.import_block(Unverified::from_rlp(b.rlp_bytes()).unwrap()) {
			panic!("error importing block which is valid by definition: {:?}", e);
		}

		last_header = view!(BlockView, &b.rlp_bytes()).header();
		db = b.drain().state.drop().1;
	}
	client.flush_queue();
	client.import_verified_blocks();
	client
}

/// Adds blocks to the client
pub fn push_blocks_to_client(client: &Arc<Client>, timestamp_salt: u64, starting_number: usize, block_number: usize) {
	let test_spec = Spec::new_test();
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

		if let Err(e) = client.import_block(Unverified::from_rlp(create_test_block(&header)).unwrap()) {
			panic!("error importing block which is valid by definition: {:?}", e);
		}
	}
}

/// Adds one block with transactions
pub fn push_block_with_transactions(client: &Arc<Client>, transactions: &[SignedTransaction]) {
	let test_spec = Spec::new_test();
	let test_engine = &*test_spec.engine;
	let block_number = client.chain_info().best_block_number as u64 + 1;

	let mut b = client.prepare_open_block(Address::default(), (0.into(), 5000000.into()), Bytes::new()).unwrap();
	b.set_timestamp(block_number * 10);

	for t in transactions {
		b.push_transaction(t.clone(), None).unwrap();
	}
	let b = b.close_and_lock().unwrap().seal(test_engine, vec![]).unwrap();

	if let Err(e) = client.import_block(Unverified::from_rlp(b.rlp_bytes()).unwrap()) {
		panic!("error importing block which is valid by definition: {:?}", e);
	}

	client.flush_queue();
	client.import_verified_blocks();
}

/// Creates dummy client (not test client) with corresponding blocks
pub fn get_test_client_with_blocks(blocks: Vec<Bytes>) -> Arc<Client> {
	let test_spec = Spec::new_test();
	let client_db = new_db();

	let client = Client::new(
		ClientConfig::default(),
		&test_spec,
		client_db,
		Arc::new(Miner::new_for_tests(&test_spec, None)),
		IoChannel::disconnected(),
	).unwrap();

	for block in blocks {
		if let Err(e) = client.import_block(Unverified::from_rlp(block).unwrap()) {
			panic!("error importing block which is well-formed: {:?}", e);
		}
	}
	client.flush_queue();
	client.import_verified_blocks();
	client
}

/// Creates new test instance of `BlockChainDB`
pub fn new_db() -> Arc<BlockChainDB> {
	struct TestBlockChainDB {
		_blooms_dir: TempDir,
		_trace_blooms_dir: TempDir,
		blooms: blooms_db::Database,
		trace_blooms: blooms_db::Database,
		key_value: Arc<KeyValueDB>,
	}

	impl BlockChainDB for TestBlockChainDB {
		fn key_value(&self) -> &Arc<KeyValueDB> {
			&self.key_value
		}

		fn blooms(&self) -> &blooms_db::Database {
			&self.blooms
		}

		fn trace_blooms(&self) -> &blooms_db::Database {
			&self.trace_blooms
		}
	}

	let blooms_dir = TempDir::new("").unwrap();
	let trace_blooms_dir = TempDir::new("").unwrap();

	let db = TestBlockChainDB {
		blooms: blooms_db::Database::open(blooms_dir.path()).unwrap(),
		trace_blooms: blooms_db::Database::open(trace_blooms_dir.path()).unwrap(),
		_blooms_dir: blooms_dir,
		_trace_blooms_dir: trace_blooms_dir,
		key_value: Arc::new(::kvdb_memorydb::create(::db::NUM_COLUMNS.unwrap()))
	};

	Arc::new(db)
}

/// Creates new instance of KeyValueDBHandler
pub fn restoration_db_handler(config: kvdb_rocksdb::DatabaseConfig) -> Box<BlockChainDBHandler> {
	struct RestorationDBHandler {
		config: kvdb_rocksdb::DatabaseConfig,
	}

	struct RestorationDB {
		blooms: blooms_db::Database,
		trace_blooms: blooms_db::Database,
		key_value: Arc<KeyValueDB>,
	}

	impl BlockChainDB for RestorationDB {
		fn key_value(&self) -> &Arc<KeyValueDB> {
			&self.key_value
		}

		fn blooms(&self) -> &blooms_db::Database {
			&self.blooms
		}

		fn trace_blooms(&self) -> &blooms_db::Database {
			&self.trace_blooms
		}
	}

	impl BlockChainDBHandler for RestorationDBHandler {
		fn open(&self, db_path: &Path) -> io::Result<Arc<BlockChainDB>> {
			let key_value = Arc::new(kvdb_rocksdb::Database::open(&self.config, &db_path.to_string_lossy())?);
			let blooms_path = db_path.join("blooms");
			let trace_blooms_path = db_path.join("trace_blooms");
			fs::create_dir_all(&blooms_path)?;
			fs::create_dir_all(&trace_blooms_path)?;
			let blooms = blooms_db::Database::open(blooms_path).unwrap();
			let trace_blooms = blooms_db::Database::open(trace_blooms_path).unwrap();
			let db = RestorationDB {
				blooms,
				trace_blooms,
				key_value,
			};
			Ok(Arc::new(db))
		}
	}

	Box::new(RestorationDBHandler { config })
}

/// Generates dummy blockchain with corresponding amount of blocks
pub fn generate_dummy_blockchain(block_number: u32) -> BlockChain {
	let db = new_db();
	let bc = BlockChain::new(BlockChainConfig::default(), &create_unverifiable_block(0, H256::zero()), db.clone());

	let mut batch = db.key_value().transaction();
	for block_order in 1..block_number {
		// Total difficulty is always 0 here.
		bc.insert_block(&mut batch, encoded::Block::new(create_unverifiable_block(block_order, bc.best_block_hash())), vec![], ExtrasInsert {
			fork_choice: ::engines::ForkChoice::New,
			is_finalized: false,
		});
		bc.commit();
	}
	db.key_value().write(batch).unwrap();
	bc
}

/// Generates dummy blockchain with corresponding amount of blocks (using creation with extra method for blocks creation)
pub fn generate_dummy_blockchain_with_extra(block_number: u32) -> BlockChain {
	let db = new_db();
	let bc = BlockChain::new(BlockChainConfig::default(), &create_unverifiable_block(0, H256::zero()), db.clone());

	let mut batch = db.key_value().transaction();
	for block_order in 1..block_number {
		// Total difficulty is always 0 here.
		bc.insert_block(&mut batch, encoded::Block::new(create_unverifiable_block_with_extra(block_order, bc.best_block_hash(), None)), vec![], ExtrasInsert {
			fork_choice: ::engines::ForkChoice::New,
			is_finalized: false,
		});
		bc.commit();
	}
	db.key_value().write(batch).unwrap();
	bc
}

/// Returns empty dummy blockchain
pub fn generate_dummy_empty_blockchain() -> BlockChain {
	let db = new_db();
	let bc = BlockChain::new(BlockChainConfig::default(), &create_unverifiable_block(0, H256::zero()), db.clone());
	bc
}

/// Returns temp state
pub fn get_temp_state() -> State<::state_db::StateDB> {
	let journal_db = get_temp_state_db();
	State::new(journal_db, U256::from(0), Default::default())
}

/// Returns temp state using coresponding factory
pub fn get_temp_state_with_factory(factory: EvmFactory) -> State<::state_db::StateDB> {
	let journal_db = get_temp_state_db();
	let mut factories = Factories::default();
	factories.vm = factory.into();
	State::new(journal_db, U256::from(0), factories)
}

/// Returns temp state db
pub fn get_temp_state_db() -> StateDB {
	let db = new_db();
	let journal_db = ::journaldb::new(db.key_value().clone(), ::journaldb::Algorithm::EarlyMerge, ::db::COL_STATE);
	StateDB::new(journal_db, 5 * 1024 * 1024)
}

/// Returns sequence of hashes of the dummy blocks
pub fn get_good_dummy_block_seq(count: usize) -> Vec<Bytes> {
	let test_spec = Spec::new_test();
	get_good_dummy_block_fork_seq(1, count, &test_spec.genesis_header().hash())
}

/// Returns sequence of hashes of the dummy blocks beginning from corresponding parent
pub fn get_good_dummy_block_fork_seq(start_number: usize, count: usize, parent_hash: &H256) -> Vec<Bytes> {
	let test_spec = Spec::new_test();
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

/// Returns hash and header of the correct dummy block
pub fn get_good_dummy_block_hash() -> (H256, Bytes) {
	let mut block_header = Header::new();
	let test_spec = Spec::new_test();
	let genesis_gas = test_spec.genesis_header().gas_limit().clone();
	block_header.set_gas_limit(genesis_gas);
	block_header.set_difficulty(U256::from(0x20000));
	block_header.set_timestamp(40);
	block_header.set_number(1);
	block_header.set_parent_hash(test_spec.genesis_header().hash());
	block_header.set_state_root(test_spec.genesis_header().state_root().clone());

	(block_header.hash(), create_test_block(&block_header))
}

/// Returns hash of the correct dummy block
pub fn get_good_dummy_block() -> Bytes {
	let (_, bytes) = get_good_dummy_block_hash();
	bytes
}

/// Returns hash of the dummy block with incorrect state root
pub fn get_bad_state_dummy_block() -> Bytes {
	let mut block_header = Header::new();
	let test_spec = Spec::new_test();
	let genesis_gas = test_spec.genesis_header().gas_limit().clone();

	block_header.set_gas_limit(genesis_gas);
	block_header.set_difficulty(U256::from(0x20000));
	block_header.set_timestamp(40);
	block_header.set_number(1);
	block_header.set_parent_hash(test_spec.genesis_header().hash());
	block_header.set_state_root(0xbad.into());

	create_test_block(&block_header)
}

/// Test actor for chain events
#[derive(Default)]
pub struct TestNotify {
	/// Messages store
	pub messages: RwLock<Vec<Bytes>>,
}

impl ChainNotify for TestNotify {
	fn broadcast(&self, message: ChainMessageType) {
		let data = match message {
			ChainMessageType::Consensus(data) => data,
			ChainMessageType::SignedPrivateTransaction(data) => data,
			ChainMessageType::PrivateTransaction(data) => data,
		};
		self.messages.write().push(data);
	}
}
