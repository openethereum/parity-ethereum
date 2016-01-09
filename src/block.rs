use util::*;
use transaction::*;
use receipt::*;
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
	fn new(state: State) -> Block {
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

/// Block that is ready for transactions to be added.
///
/// It's a bit like a Vec<Transaction>, eccept that whenever a transaction is pushed, we execute it and
/// maintain the system `state()`. We also archive execution receipts in preparation for later block creation.
pub struct OpenBlock<'engine> {
	block: Block,
	engine: &'engine Engine,
	last_hashes: LastHashes,
}

/// Just like OpenBlock, except that we've applied `Engine::on_close_block`, finished up the non-seal header fields,
/// and collected the uncles.
///
/// There is no function available to push a transaction. If you want that you'll need to `reopen()` it.
pub struct ClosedBlock<'engine> {
	open_block: OpenBlock<'engine>,
	_uncles: Vec<Header>,
}

/// A block that has a valid seal.
///
/// The block's header has valid seal arguments. The block cannot be reversed into a ClosedBlock or OpenBlock.
pub struct SealedBlock {
	block: Block,
	_bytes: Bytes,
}

impl<'engine> OpenBlock<'engine> {
	/// Create a new OpenBlock ready for transaction pushing.
	pub fn new<'a>(engine: &'a Engine, db: OverlayDB, parent: &Header, last_hashes: LastHashes) -> OpenBlock<'a> {
		let mut r = OpenBlock {
			block: Block::new(State::from_existing(db, parent.state_root.clone(), engine.account_start_nonce())),
			engine: engine,
			last_hashes: last_hashes,
		};

		engine.populate_from_parent(&mut r.block.header, parent);
		engine.on_new_block(&mut r.block);
		r
	}

	/// Get the environment info concerning this block.
	pub fn env_info(&self) -> EnvInfo {
		// TODO: memoise.
		EnvInfo {
			number: self.block.header.number.clone(),
			author: self.block.header.author.clone(),
			timestamp: self.block.header.timestamp.clone(),
			difficulty: self.block.header.difficulty.clone(),
			last_hashes: self.last_hashes.clone(),
			gas_used: if let Some(ref t) = self.block.archive.last() {t.receipt.gas_used} else {U256::from(0)},
			gas_limit: self.block.header.gas_limit.clone(),
		}
	}

	/// Push a transaction into the block. It will be executed, and archived together with the receipt.
	pub fn push_transaction(&mut self, t: Transaction, h: Option<H256>) -> Result<&Receipt, EthcoreError> {
		let env_info = self.env_info();
		match self.block.state.apply(&env_info, self.engine, &t, true) {
			Ok(x) => {
				self.block.archive_set.insert(h.unwrap_or_else(||t.sha3()));
				self.block.archive.push(Entry { transaction: t, receipt: x.receipt });
				Ok(&self.block.archive.last().unwrap().receipt)
			}
			Err(x) => Err(x)
		}
	}

	/// Turn this into a `ClosedBlock`. A BlockChain must be provided in order to figure ou the uncles.
	pub fn close(self, _uncles: Vec<Header>) -> ClosedBlock<'engine> { unimplemented!(); }
}

impl<'engine> IsBlock for OpenBlock<'engine> {
	fn block(&self) -> &Block { &self.block }
}

impl<'engine> ClosedBlock<'engine> {
	/// Get the hash of the header without seal arguments.
	pub fn preseal_hash(&self) -> H256 { unimplemented!(); }

	/// Turn this into a `ClosedBlock`. A BlockChain must be provided in order to figure ou the uncles.
	pub fn seal(self, _seal_fields: Vec<Bytes>) -> SealedBlock { unimplemented!(); }

	/// Turn this back into an `OpenBlock`.
	pub fn reopen(self) -> OpenBlock<'engine> { unimplemented!(); }
}

impl<'engine> IsBlock for ClosedBlock<'engine> {
	fn block(&self) -> &Block { &self.open_block.block }
}

impl SealedBlock {
}

impl IsBlock for SealedBlock {
	fn block(&self) -> &Block { &self.block }
}

#[test]
fn open_block() {
	use super::*;
	use spec::*;
	let engine = Spec::new_test().to_engine().unwrap();
	let genesis_header = engine.spec().genesis_header();
	let mut db = OverlayDB::new_temp();
	engine.spec().ensure_db_good(&mut db);
	let b = OpenBlock::new(engine.deref(), db, &genesis_header, vec![genesis_header.hash()]);
}
