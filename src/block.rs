use std::collections::hash_set::*;
use util::hash::*;
use util::error::*;
use transaction::*;
use engine::*;
use header::*;
use env_info::*;
use state::*;

/// A transaction/receipt execution entry.
pub struct Entry {
	transaction: Transaction,
	receipt: Receipt,
}

/// Internal type for a block's common elements.
pub struct Block {
	header: Header,

	/// State is the most final state in the block.
	state: State,

	archive: Vec<Entry>,
	archive_set: HashSet<H256>,
}

impl Block {
	fn new(header: Header, state: State) -> Block {
		Block {
			header: Header::new(),
			state: state,
			archive: Vec::new(),
			archive_set: HashSet::new(),
		}
	}

	pub fn state_mut(&mut self) -> &mut State { &mut self.state }
}

/// Trait for a object that is_a `Block`.
pub trait IsBlock {
	/// Get the block associated with this object.
	fn block(&self) -> &Block;

	/// Get the header associated with this object's block.
	fn header(&self) -> &Header { &self.block().header }

	/// Get the final state associated with this object's block.
	fn state(&self) -> &State { &self.block().state }

	/// Get all information on transactions in this block.
	fn archive(&self) -> &Vec<Entry> { &self.block().archive }
}

impl IsBlock for Block {
	fn block(&self) -> &Block { self }
}
/*
/// Block that is ready for transactions to be added.
///
/// It's a bit like a Vec<Transaction>, eccept that whenever a transaction is pushed, we execute it and
/// maintain the system `state()`. We also archive execution receipts in preparation for later block creation.
pub struct OpenBlock {
	block: Block,
	engine: &Engine,
	last_hashes: LastHashes,
}

/// Just like OpenBlock, except that we've applied `Engine::on_close_block`, finished up the non-seal header fields,
/// and collected the uncles.
///
/// There is no function available to push a transaction. If you want that you'll need to `reopen()` it.
pub struct ClosedBlock {
	open_block: OpenBlock,
	uncles: Vec<Header>,
}

/// A block that has a valid seal.
///
/// The block's header has valid seal arguments. The block cannot be reversed into a ClosedBlock or OpenBlock.
pub struct SealedBlock {
	block: Block,
	bytes: Bytes,
}

impl OpenBlock {
	pub fn new(engine: &Engine, mut db: OverlayDB, parent: &Header, last_hashes: LastHashes) -> OpenBlock {
		let mut r = OpenBlock {
			block: Block::new(State::new_existing(db, parent.state_root.clone(), engine.account_start_nonce())),
			engine: engine,
			last_hashes: last_hashes,
		}

		engine.populate_from_parent(r.block.header, parent);
		engine.on_init_block(&mut r);
		r
	}

	/// Turn this into a `ClosedBlock`. A BlockChain must be provided in order to figure ou the uncles.
	pub fn push_transaction(&mut self, t: Transaction, mut h: Option<H256>) -> Result<&Receipt, EthcoreError> {
		let env_info = EnvInfo{
			number: self.header.number,
			author: self.header.author,
			timestamp: self.header.timestamp,
			difficulty: self.header.difficulty,
			last_hashes: self.last_hashes.clone(),
			gas_used: if let Some(ref t) = self.archive.last() {t.receipt.gas_used} else {U256::from(0)},
		};
		match self.state.apply(env_info, self.engine, t, true) {
			Ok(x) => {
				self.transactionHashes.insert(h.unwrap_or_else(||t.sha3()));
				self.transactions.push(BlockTransaction{t, x.receipt});
				Ok(&self.transactions.last().unwrap().receipt)
			}
			Err(x) => Err(x)
		}
	}

	/// Turn this into a `ClosedBlock`. A BlockChain must be provided in order to figure ou the uncles.
	pub fn close(self, bc: &BlockChain) -> ClosedBlock { unimplemented!(); }
}

impl IsBlock for OpenBlock {
	fn block(&self) -> &Block { self.block }
	fn block_mut(&self) -> &mut Block { self.block }
}

impl ClosedBlock {
	/// Get the hash of the header without seal arguments.
	pub fn preseal_hash(&self) -> H256 { unimplemented!(); }

	/// Turn this into a `ClosedBlock`. A BlockChain must be provided in order to figure ou the uncles.
	pub fn seal(self, seal_fields: Vec<Bytes>) -> SealedBlock { unimplemented!(); }

	/// Turn this back into an `OpenBlock`.
	pub fn reopen(self) -> OpenBlock { unimplemented!(); }
}

impl IsBlock for ClosedBlock {
	fn block(&self) -> &Block { self.open_block.block }
	fn block_mut(&self) -> &mut Block { self.open_block.block }
}

impl SealedBlock {
}

impl IsBlock for SealedBlock {
	fn block(&self) -> &Block { self.block }
	fn block_mut(&self) -> &mut Block { self.block.block }
}
*/