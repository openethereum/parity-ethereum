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

//! Test client.

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrder};
use util::*;
use transaction::{Transaction, LocalizedTransaction, SignedTransaction, Action};
use blockchain::TreeRoute;
use client::{BlockChainClient, MiningBlockChainClient, BlockChainInfo, BlockStatus, BlockID, TransactionID, UncleID, TraceId, TraceFilter, LastHashes, CallAnalytics};
use header::{Header as BlockHeader, BlockNumber};
use filter::Filter;
use log_entry::LocalizedLogEntry;
use receipt::{Receipt, LocalizedReceipt};
use blockchain::extras::BlockReceipts;
use error::{ImportResult};
use evm::Factory as EvmFactory;
use miner::{Miner, MinerService};

use block_queue::BlockQueueInfo;
use block::{SealedBlock, LockedBlock, OpenBlock};
use executive::Executed;
use error::{ExecutionError};
use trace::LocalizedTrace;

use miner::{TransactionImportResult, AccountDetails};
use error::Error as EthError;

/// Test client.
pub struct TestBlockChainClient {
	/// Blocks.
	pub blocks: RwLock<HashMap<H256, Bytes>>,
	/// Mapping of numbers to hashes.
 	pub numbers: RwLock<HashMap<usize, H256>>,
	/// Genesis block hash.
	pub genesis_hash: H256,
	/// Last block hash.
	pub last_hash: RwLock<H256>,
	/// Difficulty.
	pub difficulty: RwLock<U256>,
	/// Balances.
	pub balances: RwLock<HashMap<Address, U256>>,
	/// Nonces.
	pub nonces: RwLock<HashMap<Address, U256>>,
	/// Storage.
	pub storage: RwLock<HashMap<(Address, H256), H256>>,
	/// Code.
	pub code: RwLock<HashMap<Address, Bytes>>,
	/// Execution result.
	pub execution_result: RwLock<Option<Executed>>,
	/// Transaction receipts.
	pub receipts: RwLock<HashMap<TransactionID, LocalizedReceipt>>,
	/// Block queue size.
	pub queue_size: AtomicUsize,
	/// Miner
	pub miner: Arc<Miner>,
}

#[derive(Clone)]
/// Used for generating test client blocks.
pub enum EachBlockWith {
	/// Plain block.
	Nothing,
	/// Block with an uncle.
	Uncle,
	/// Block with a transaction.
	Transaction,
	/// Block with an uncle and transaction.
	UncleAndTransaction
}

impl Default for TestBlockChainClient {
	fn default() -> Self {
		TestBlockChainClient::new()
	}
}

impl TestBlockChainClient {
	/// Creates new test client.
	pub fn new() -> Self {

		let mut client = TestBlockChainClient {
			blocks: RwLock::new(HashMap::new()),
			numbers: RwLock::new(HashMap::new()),
			genesis_hash: H256::new(),
			last_hash: RwLock::new(H256::new()),
			difficulty: RwLock::new(From::from(0)),
			balances: RwLock::new(HashMap::new()),
			nonces: RwLock::new(HashMap::new()),
			storage: RwLock::new(HashMap::new()),
			code: RwLock::new(HashMap::new()),
			execution_result: RwLock::new(None),
			receipts: RwLock::new(HashMap::new()),
			queue_size: AtomicUsize::new(0),
			miner: Arc::new(Miner::default()),
		};
		client.add_blocks(1, EachBlockWith::Nothing); // add genesis block
		client.genesis_hash = client.last_hash.read().unwrap().clone();
		client
	}

	/// Set the transaction receipt result
	pub fn set_transaction_receipt(&self, id: TransactionID, receipt: LocalizedReceipt) {
		self.receipts.write().unwrap().insert(id, receipt);
	}

	/// Set the execution result.
	pub fn set_execution_result(&self, result: Executed) {
		*self.execution_result.write().unwrap() = Some(result);
	}

	/// Set the balance of account `address` to `balance`.
	pub fn set_balance(&self, address: Address, balance: U256) {
		self.balances.write().unwrap().insert(address, balance);
	}

	/// Set nonce of account `address` to `nonce`.
	pub fn set_nonce(&self, address: Address, nonce: U256) {
		self.nonces.write().unwrap().insert(address, nonce);
	}

	/// Set `code` at `address`.
	pub fn set_code(&self, address: Address, code: Bytes) {
		self.code.write().unwrap().insert(address, code);
	}

	/// Set storage `position` to `value` for account `address`.
	pub fn set_storage(&self, address: Address, position: H256, value: H256) {
		self.storage.write().unwrap().insert((address, position), value);
	}

	/// Set block queue size for testing
	pub fn set_queue_size(&self, size: usize) {
		self.queue_size.store(size, AtomicOrder::Relaxed);
	}

	/// Add blocks to test client.
	pub fn add_blocks(&self, count: usize, with: EachBlockWith) {
		let len = self.numbers.read().unwrap().len();
		for n in len..(len + count) {
			let mut header = BlockHeader::new();
			header.difficulty = From::from(n);
			header.parent_hash = self.last_hash.read().unwrap().clone();
			header.number = n as BlockNumber;
			header.gas_limit = U256::from(1_000_000);
			let uncles = match with {
				EachBlockWith::Uncle | EachBlockWith::UncleAndTransaction => {
					let mut uncles = RlpStream::new_list(1);
					let mut uncle_header = BlockHeader::new();
					uncle_header.difficulty = From::from(n);
					uncle_header.parent_hash = self.last_hash.read().unwrap().clone();
					uncle_header.number = n as BlockNumber;
					uncles.append(&uncle_header);
					header.uncles_hash = uncles.as_raw().sha3();
					uncles
				},
				_ => RlpStream::new_list(0)
			};
			let txs = match with {
				EachBlockWith::Transaction | EachBlockWith::UncleAndTransaction => {
					let mut txs = RlpStream::new_list(1);
					let keypair = KeyPair::create().unwrap();
					// Update nonces value
					self.nonces.write().unwrap().insert(keypair.address(), U256::one());
					let tx = Transaction {
						action: Action::Create,
						value: U256::from(100),
						data: "3331600055".from_hex().unwrap(),
						gas: U256::from(100_000),
						gas_price: U256::one(),
						nonce: U256::zero()
					};
					let signed_tx = tx.sign(&keypair.secret());
					txs.append(&signed_tx);
					txs.out()
				},
				_ => rlp::NULL_RLP.to_vec()
			};

			let mut rlp = RlpStream::new_list(3);
			rlp.append(&header);
			rlp.append_raw(&txs, 1);
			rlp.append_raw(uncles.as_raw(), 1);
			self.import_block(rlp.as_raw().to_vec()).unwrap();
		}
	}

	/// Make a bad block by setting invalid extra data.
	pub fn corrupt_block(&mut self, n: BlockNumber) {
		let hash = self.block_hash(BlockID::Number(n)).unwrap();
		let mut header: BlockHeader = decode(&self.block_header(BlockID::Number(n)).unwrap());
		header.extra_data = b"This extra data is way too long to be considered valid".to_vec();
		let mut rlp = RlpStream::new_list(3);
		rlp.append(&header);
		rlp.append_raw(&rlp::NULL_RLP, 1);
		rlp.append_raw(&rlp::NULL_RLP, 1);
		self.blocks.write().unwrap().insert(hash, rlp.out());
	}

	/// Make a bad block by setting invalid parent hash.
	pub fn corrupt_block_parent(&mut self, n: BlockNumber) {
		let hash = self.block_hash(BlockID::Number(n)).unwrap();
		let mut header: BlockHeader = decode(&self.block_header(BlockID::Number(n)).unwrap());
		header.parent_hash = H256::from(42);
		let mut rlp = RlpStream::new_list(3);
		rlp.append(&header);
		rlp.append_raw(&rlp::NULL_RLP, 1);
		rlp.append_raw(&rlp::NULL_RLP, 1);
		self.blocks.write().unwrap().insert(hash, rlp.out());
	}

	/// TODO:
	pub fn block_hash_delta_minus(&mut self, delta: usize) -> H256 {
		let blocks_read = self.numbers.read().unwrap();
		let index = blocks_read.len() - delta;
		blocks_read[&index].clone()
	}

	fn block_hash(&self, id: BlockID) -> Option<H256> {
		match id {
			BlockID::Hash(hash) => Some(hash),
			BlockID::Number(n) => self.numbers.read().unwrap().get(&(n as usize)).cloned(),
			BlockID::Earliest => self.numbers.read().unwrap().get(&0).cloned(),
			BlockID::Latest => self.numbers.read().unwrap().get(&(self.numbers.read().unwrap().len() - 1)).cloned()
		}
	}
}

impl MiningBlockChainClient for TestBlockChainClient {
	fn try_seal(&self, block: LockedBlock, _seal: Vec<Bytes>) -> Result<SealedBlock, LockedBlock> {
		Err(block)
	}


	fn prepare_open_block(&self, _author: Address, _gas_floor_target: U256, _extra_data: Bytes) -> OpenBlock {
		unimplemented!();
	}
}

impl BlockChainClient for TestBlockChainClient {
	fn call(&self, _t: &SignedTransaction, _analytics: CallAnalytics) -> Result<Executed, ExecutionError> {
		Ok(self.execution_result.read().unwrap().clone().unwrap())
	}

	fn block_total_difficulty(&self, _id: BlockID) -> Option<U256> {
		Some(U256::zero())
	}

	fn block_hash(&self, id: BlockID) -> Option<H256> {
		Self::block_hash(self, id)
	}

	fn nonce(&self, address: &Address, id: BlockID) -> Option<U256> {
		match id {
			BlockID::Latest => Some(self.nonces.read().unwrap().get(address).cloned().unwrap_or_else(U256::zero)),
			_ => None,
		}
	}

	fn code(&self, address: &Address) -> Option<Bytes> {
		self.code.read().unwrap().get(address).cloned()
	}

	fn balance(&self, address: &Address, id: BlockID) -> Option<U256> {
		if let BlockID::Latest = id {
			Some(self.balances.read().unwrap().get(address).cloned().unwrap_or_else(U256::zero))
		} else {
			None
		}
	}

	fn storage_at(&self, address: &Address, position: &H256, id: BlockID) -> Option<H256> {
		if let BlockID::Latest = id {
			Some(self.storage.read().unwrap().get(&(address.clone(), position.clone())).cloned().unwrap_or_else(H256::new))
		} else {
			None
		}
	}

	fn transaction(&self, _id: TransactionID) -> Option<LocalizedTransaction> {
		unimplemented!();
	}

	fn uncle(&self, _id: UncleID) -> Option<BlockHeader> {
		unimplemented!();
	}

	fn transaction_receipt(&self, id: TransactionID) -> Option<LocalizedReceipt> {
		self.receipts.read().unwrap().get(&id).cloned()
	}

	fn blocks_with_bloom(&self, _bloom: &H2048, _from_block: BlockID, _to_block: BlockID) -> Option<Vec<BlockNumber>> {
		unimplemented!();
	}

	fn logs(&self, _filter: Filter) -> Vec<LocalizedLogEntry> {
		unimplemented!();
	}

	fn last_hashes(&self) -> LastHashes {
		unimplemented!();
	}

	fn block_header(&self, id: BlockID) -> Option<Bytes> {
		self.block_hash(id).and_then(|hash| self.blocks.read().unwrap().get(&hash).map(|r| Rlp::new(r).at(0).as_raw().to_vec()))
	}

	fn block_body(&self, id: BlockID) -> Option<Bytes> {
		self.block_hash(id).and_then(|hash| self.blocks.read().unwrap().get(&hash).map(|r| {
			let mut stream = RlpStream::new_list(2);
			stream.append_raw(Rlp::new(&r).at(1).as_raw(), 1);
			stream.append_raw(Rlp::new(&r).at(2).as_raw(), 1);
			stream.out()
		}))
	}

	fn block(&self, id: BlockID) -> Option<Bytes> {
		self.block_hash(id).and_then(|hash| self.blocks.read().unwrap().get(&hash).cloned())
	}

	fn block_status(&self, id: BlockID) -> BlockStatus {
		match id {
			BlockID::Number(number) if (number as usize) < self.blocks.read().unwrap().len() => BlockStatus::InChain,
			BlockID::Hash(ref hash) if self.blocks.read().unwrap().get(hash).is_some() => BlockStatus::InChain,
			_ => BlockStatus::Unknown
		}
	}

	// works only if blocks are one after another 1 -> 2 -> 3
	fn tree_route(&self, from: &H256, to: &H256) -> Option<TreeRoute> {
		Some(TreeRoute {
			ancestor: H256::new(),
			index: 0,
			blocks: {
				let numbers_read = self.numbers.read().unwrap();
				let mut adding = false;

				let mut blocks = Vec::new();
				for (_, hash) in numbers_read.iter().sort_by(|tuple1, tuple2| tuple1.0.cmp(tuple2.0)) {
					if hash == to {
						if adding {
							blocks.push(hash.clone());
						}
						adding = false;
						break;
					}
					if hash == from {
						adding = true;
					}
					if adding {
						blocks.push(hash.clone());
					}
				}
				if adding { Vec::new() } else { blocks }
			}
		})
	}

	fn find_uncles(&self, _hash: &H256) -> Option<Vec<H256>> {
		None
	}

	// TODO: returns just hashes instead of node state rlp(?)
	fn state_data(&self, hash: &H256) -> Option<Bytes> {
		// starts with 'f' ?
		if *hash > H256::from("f000000000000000000000000000000000000000000000000000000000000000") {
			let mut rlp = RlpStream::new();
			rlp.append(&hash.clone());
			return Some(rlp.out());
		}
		None
	}

	fn block_receipts(&self, hash: &H256) -> Option<Bytes> {
		// starts with 'f' ?
		if *hash > H256::from("f000000000000000000000000000000000000000000000000000000000000000") {
			let receipt = BlockReceipts::new(vec![Receipt::new(
				H256::zero(),
				U256::zero(),
				vec![])]);
			let mut rlp = RlpStream::new();
			rlp.append(&receipt);
			return Some(rlp.out());
		}
		None
	}

	fn import_block(&self, b: Bytes) -> ImportResult {
		let header = Rlp::new(&b).val_at::<BlockHeader>(0);
		let h = header.hash();
		let number: usize = header.number as usize;
		if number > self.blocks.read().unwrap().len() {
			panic!("Unexpected block number. Expected {}, got {}", self.blocks.read().unwrap().len(), number);
		}
		if number > 0 {
			match self.blocks.read().unwrap().get(&header.parent_hash) {
				Some(parent) => {
					let parent = Rlp::new(parent).val_at::<BlockHeader>(0);
					if parent.number != (header.number - 1) {
						panic!("Unexpected block parent");
					}
				},
				None => {
					panic!("Unknown block parent {:?} for block {}", header.parent_hash, number);
				}
			}
		}
		let len = self.numbers.read().unwrap().len();
		if number == len {
			{
				let mut difficulty = self.difficulty.write().unwrap();
				*difficulty.deref_mut() = *difficulty.deref() + header.difficulty;
			}
			mem::replace(self.last_hash.write().unwrap().deref_mut(), h.clone());
			self.blocks.write().unwrap().insert(h.clone(), b);
			self.numbers.write().unwrap().insert(number, h.clone());
			let mut parent_hash = header.parent_hash;
			if number > 0 {
				let mut n = number - 1;
				while n > 0 && self.numbers.read().unwrap()[&n] != parent_hash {
					*self.numbers.write().unwrap().get_mut(&n).unwrap() = parent_hash.clone();
					n -= 1;
					parent_hash = Rlp::new(&self.blocks.read().unwrap()[&parent_hash]).val_at::<BlockHeader>(0).parent_hash;
				}
			}
		}
		else {
			self.blocks.write().unwrap().insert(h.clone(), b.to_vec());
		}
		Ok(h)
	}

	fn queue_info(&self) -> BlockQueueInfo {
		BlockQueueInfo {
			verified_queue_size: self.queue_size.load(AtomicOrder::Relaxed),
			unverified_queue_size: 0,
			verifying_queue_size: 0,
			max_queue_size: 0,
			max_mem_use: 0,
			mem_used: 0,
		}
	}

	fn clear_queue(&self) {
	}

	fn chain_info(&self) -> BlockChainInfo {
		BlockChainInfo {
			total_difficulty: *self.difficulty.read().unwrap(),
			pending_total_difficulty: *self.difficulty.read().unwrap(),
			genesis_hash: self.genesis_hash.clone(),
			best_block_hash: self.last_hash.read().unwrap().clone(),
			best_block_number: self.blocks.read().unwrap().len() as BlockNumber - 1,
		}
	}

	fn vm_factory(&self) -> &EvmFactory {
		unimplemented!();
	}

	fn filter_traces(&self, _filter: TraceFilter) -> Option<Vec<LocalizedTrace>> {
		unimplemented!();
	}

	fn trace(&self, _trace: TraceId) -> Option<LocalizedTrace> {
		unimplemented!();
	}

	fn transaction_traces(&self, _trace: TransactionID) -> Option<Vec<LocalizedTrace>> {
		unimplemented!();
	}

	fn block_traces(&self, _trace: BlockID) -> Option<Vec<LocalizedTrace>> {
		unimplemented!();
	}

	fn import_transactions(&self, transactions: Vec<SignedTransaction>) -> Vec<Result<TransactionImportResult, EthError>> {
		let nonces = self.nonces.read().unwrap();
		let balances = self.balances.read().unwrap();
		let fetch_account = |a: &Address| AccountDetails {
			nonce: nonces[a],
			balance: balances[a],
		};

		self.miner.import_transactions(transactions, &fetch_account)
	}

	fn all_transactions(&self) -> Vec<SignedTransaction> {
		self.miner.all_transactions()
	}

	fn take_snapshot(&self, _root_dir: &Path) {
		unimplemented!()
	}
}
