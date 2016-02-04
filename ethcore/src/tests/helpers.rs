#[cfg(feature = "json-tests")]
use client::{BlockChainClient, Client};
use std::env;
use common::*;
use std::path::PathBuf;
use spec::*;
use std::fs::{remove_dir_all};
use blockchain::{BlockChain};
use state::*;
use rocksdb::*;


#[cfg(feature = "json-tests")]
pub enum ChainEra {
	Frontier,
	Homestead,
}

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

	pub fn as_str(&self) -> &str {
		self.path.to_str().unwrap()
	}
}

impl Drop for RandomTempPath {
	fn drop(&mut self) {
		if let Err(e) = remove_dir_all(self.as_path()) {
			panic!("failed to remove temp directory, probably something failed to destroyed ({})", e);
		}
	}
}

#[cfg(test)]
pub struct GuardedTempResult<T> {
	result: T,
	_temp: RandomTempPath
}

impl<T> GuardedTempResult<T> {
    pub fn reference(&self) -> &T {
        &self.result
    }

    pub fn reference_mut(&mut self) -> &mut T {
    	&mut self.result
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

fn create_unverifiable_block_header(order: u32, parent_hash: H256) -> Header {
	let mut header = Header::new();
	header.gas_limit = x!(0);
	header.difficulty = x!(order * 100);
	header.timestamp = (order * 10) as u64;
	header.number = order as u64;
	header.parent_hash = parent_hash;
	header.state_root = H256::zero();

	header
}

fn create_unverifiable_block_with_extra(order: u32, parent_hash: H256, extra: Option<Bytes>) -> Bytes {
	let mut header = create_unverifiable_block_header(order, parent_hash);
	header.extra_data = match extra {
		Some(extra_data) => extra_data,
		None => {
			let base = (order & 0x000000ff) as u8;
			let generated: Vec<u8> = vec![base + 1, base + 2, base + 3];
			generated
		}
	};
	create_test_block(&header)
}

fn create_unverifiable_block(order: u32, parent_hash: H256) -> Bytes {
	create_test_block(&create_unverifiable_block_header(order, parent_hash))
}

pub fn create_test_block_with_data(header: &Header, transactions: &[&Transaction], uncles: &[Header]) -> Bytes {
	let mut rlp = RlpStream::new_list(3);
	rlp.append(header);
	rlp.begin_list(transactions.len());
	for t in transactions {
		rlp.append_raw(&t.rlp_bytes_opt(Seal::With), 1);
	}
	rlp.append(&uncles);
	rlp.out()
}

#[cfg(feature = "json-tests")]
pub fn generate_dummy_client(block_number: u32) -> GuardedTempResult<Arc<Client>> {
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

	GuardedTempResult::<Arc<Client>> {
		_temp: dir,
		result: client
	}
}

#[cfg(feature = "json-tests")]
pub fn get_test_client_with_blocks(blocks: Vec<Bytes>) -> GuardedTempResult<Arc<Client>> {
	let dir = RandomTempPath::new();
	let client = Client::new(get_test_spec(), dir.as_path(), IoChannel::disconnected()).unwrap();
	for block in &blocks {
		if let Err(_) = client.import_block(block.clone()) {
			panic!("panic importing block which is well-formed");
		}
	}
	client.flush_queue();
	client.import_verified_blocks(&IoChannel::disconnected());

	GuardedTempResult::<Arc<Client>> {
		_temp: dir,
		result: client
	}
}

pub fn generate_dummy_blockchain(block_number: u32) -> GuardedTempResult<BlockChain> {
	let temp = RandomTempPath::new();
	let bc = BlockChain::new(&create_unverifiable_block(0, H256::zero()), temp.as_path());
	for block_order in 1..block_number {
		bc.insert_block(&create_unverifiable_block(block_order, bc.best_block_hash()));
	}

	GuardedTempResult::<BlockChain> {
		_temp: temp,
		result: bc
	}
}

pub fn generate_dummy_blockchain_with_extra(block_number: u32) -> GuardedTempResult<BlockChain> {
	let temp = RandomTempPath::new();
	let bc = BlockChain::new(&create_unverifiable_block(0, H256::zero()), temp.as_path());
	for block_order in 1..block_number {
		bc.insert_block(&create_unverifiable_block_with_extra(block_order, bc.best_block_hash(), None));
	}

	GuardedTempResult::<BlockChain> {
		_temp: temp,
		result: bc
	}
}

pub fn generate_dummy_empty_blockchain() -> GuardedTempResult<BlockChain> {
	let temp = RandomTempPath::new();
	let bc = BlockChain::new(&create_unverifiable_block(0, H256::zero()), temp.as_path());

	GuardedTempResult::<BlockChain> {
		_temp: temp,
		result: bc
	}
}

pub fn get_temp_journal_db() -> GuardedTempResult<JournalDB> {
	let temp = RandomTempPath::new();
	let db = DB::open_default(temp.as_str()).unwrap();
	let journal_db = JournalDB::new(db);
	GuardedTempResult {
		_temp: temp,
		result: journal_db
	}
}

pub fn get_temp_state() -> GuardedTempResult<State> {
	let temp = RandomTempPath::new();
	let journal_db = get_temp_journal_db_in(temp.as_path());
	GuardedTempResult {
	    _temp: temp,
		result: State::new(journal_db, U256::from(0u8))
	}
}

pub fn get_temp_journal_db_in(path: &Path) -> JournalDB {
	let db = DB::open_default(path.to_str().unwrap()).unwrap();
	JournalDB::new(db)
}

pub fn get_temp_state_in(path: &Path) -> State {
	let journal_db = get_temp_journal_db_in(path);
	State::new(journal_db, U256::from(0u8))
}

pub fn get_good_dummy_block() -> Bytes {
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

#[cfg(feature = "json-tests")]
pub fn get_bad_state_dummy_block() -> Bytes {
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
