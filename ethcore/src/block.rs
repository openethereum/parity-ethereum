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

//! Blockchain block.

#![cfg_attr(feature="dev", allow(ptr_arg))] // Because of &LastHashes -> &Vec<_>

use common::*;
use engine::*;
use state::*;
use verification::PreverifiedBlock;
use trace::Trace;

/// A block, encoded as it is on the block chain.
#[derive(Default, Debug, Clone)]
pub struct Block {
	/// The header of this block.
	pub header: Header,
	/// The transactions in this block.
	pub transactions: Vec<SignedTransaction>,
	/// The uncles of this block.
	pub uncles: Vec<Header>,
}

impl Block {
	/// Returns true if the given bytes form a valid encoding of a block in RLP.
	// TODO: implement Decoder for this and have this use that.
	pub fn is_good(b: &[u8]) -> bool {
		/*
		let urlp = UntrustedRlp::new(&b);
		if !urlp.is_list() || urlp.item_count() != 3 || urlp.size() != b.len() { return false; }
		if urlp.val_at::<Header>(0).is_err() { return false; }

		if !urlp.at(1).unwrap().is_list() { return false; }
		if urlp.at(1).unwrap().iter().find(|i| i.as_val::<Transaction>().is_err()).is_some() {
			return false;
		}

		if !urlp.at(2).unwrap().is_list() { return false; }
		if urlp.at(2).unwrap().iter().find(|i| i.as_val::<Header>().is_err()).is_some() {
			return false;
		}
		true*/
		UntrustedRlp::new(b).as_val::<Block>().is_ok()
	}
}

impl Decodable for Block {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		if decoder.as_raw().len() != try!(decoder.as_rlp().payload_info()).total() {
			return Err(DecoderError::RlpIsTooBig);
		}
		let d = decoder.as_rlp();
		if d.item_count() != 3 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
		Ok(Block {
			header: try!(d.val_at(0)),
			transactions: try!(d.val_at(1)),
			uncles: try!(d.val_at(2)),
		})
	}
}

/// Internal type for a block's common elements.
#[derive(Clone)]
pub struct ExecutedBlock {
	base: Block,

	receipts: Vec<Receipt>,
	transactions_set: HashSet<H256>,
	state: State,
	traces: Option<Vec<Trace>>,
}

/// A set of references to `ExecutedBlock` fields that are publicly accessible.
pub struct BlockRefMut<'a> {
	/// Block header.
	pub header: &'a Header,
	/// Block transactions.
	pub transactions: &'a Vec<SignedTransaction>,
	/// Block uncles.
	pub uncles: &'a Vec<Header>,
	/// Transaction receipts.
	pub receipts: &'a Vec<Receipt>,
	/// State.
	pub state: &'a mut State,
	/// Traces.
	pub traces: &'a Option<Vec<Trace>>,
}

/// A set of immutable references to `ExecutedBlock` fields that are publicly accessible.
pub struct BlockRef<'a> {
	/// Block header.
	pub header: &'a Header,
	/// Block transactions.
	pub transactions: &'a Vec<SignedTransaction>,
	/// Block uncles.
	pub uncles: &'a Vec<Header>,
	/// Transaction receipts.
	pub receipts: &'a Vec<Receipt>,
	/// State.
	pub state: &'a State,
	/// Traces.
	pub traces: &'a Option<Vec<Trace>>,
}

impl ExecutedBlock {
	/// Create a new block from the given `state`.
	fn new(state: State, tracing: bool) -> ExecutedBlock {
		ExecutedBlock {
			base: Default::default(),
			receipts: Default::default(),
			transactions_set: Default::default(),
			state: state,
			traces: if tracing {Some(Vec::new())} else {None},
		}
	}

	/// Get a structure containing individual references to all public fields.
	pub fn fields_mut(&mut self) -> BlockRefMut {
		BlockRefMut {
			header: &self.base.header,
			transactions: &self.base.transactions,
			uncles: &self.base.uncles,
			state: &mut self.state,
			receipts: &self.receipts,
			traces: &self.traces,
		}
	}

	/// Get a structure containing individual references to all public fields.
	pub fn fields(&self) -> BlockRef {
		BlockRef {
			header: &self.base.header,
			transactions: &self.base.transactions,
			uncles: &self.base.uncles,
			state: &self.state,
			receipts: &self.receipts,
			traces: &self.traces,
		}
	}
}

/// Trait for a object that is a `ExecutedBlock`.
pub trait IsBlock {
	/// Get the block associated with this object.
	fn block(&self) -> &ExecutedBlock;

	/// Get the header associated with this object's block.
	fn header(&self) -> &Header { &self.block().base.header }

	/// Get the final state associated with this object's block.
	fn state(&self) -> &State { &self.block().state }

	/// Get all information on transactions in this block.
	fn transactions(&self) -> &Vec<SignedTransaction> { &self.block().base.transactions }

	/// Get all information on receipts in this block.
	fn receipts(&self) -> &Vec<Receipt> { &self.block().receipts }

	/// Get all information concerning transaction tracing in this block.
	fn traces(&self) -> &Option<Vec<Trace>> { &self.block().traces }

	/// Get all uncles in this block.
	fn uncles(&self) -> &Vec<Header> { &self.block().base.uncles }
}

impl IsBlock for ExecutedBlock {
	fn block(&self) -> &ExecutedBlock { self }
}

/// Block that is ready for transactions to be added.
///
/// It's a bit like a Vec<Transaction>, except that whenever a transaction is pushed, we execute it and
/// maintain the system `state()`. We also archive execution receipts in preparation for later block creation.
pub struct OpenBlock<'x> {
	block: ExecutedBlock,
	engine: &'x Engine,
	last_hashes: LastHashes,
}

/// Just like `OpenBlock`, except that we've applied `Engine::on_close_block`, finished up the non-seal header fields,
/// and collected the uncles.
///
/// There is no function available to push a transaction.
#[derive(Clone)]
pub struct ClosedBlock {
	block: ExecutedBlock,
	uncle_bytes: Bytes,
	last_hashes: LastHashes,
	unclosed_state: State,
}

/// Just like `ClosedBlock` except that we can't reopen it and it's faster.
///
/// We actually store the post-`Engine::on_close_block` state, unlike in `ClosedBlock` where it's the pre.
#[derive(Clone)]
pub struct LockedBlock {
	block: ExecutedBlock,
	uncle_bytes: Bytes,
	last_hashes: LastHashes,
}

/// A block that has a valid seal.
///
/// The block's header has valid seal arguments. The block cannot be reversed into a `ClosedBlock` or `OpenBlock`.
pub struct SealedBlock {
	block: ExecutedBlock,
	uncle_bytes: Bytes,
}

impl<'x> OpenBlock<'x> {
	#[cfg_attr(feature="dev", allow(too_many_arguments))]
	/// Create a new `OpenBlock` ready for transaction pushing.
	pub fn new(engine: &'x Engine, tracing: bool, db: Box<JournalDB>, parent: &Header, last_hashes: LastHashes, author: Address, gas_floor_target: U256, extra_data: Bytes) -> Self {
		let mut r = OpenBlock {
			block: ExecutedBlock::new(State::from_existing(db, parent.state_root().clone(), engine.account_start_nonce()), tracing),
			engine: engine,
			last_hashes: last_hashes,
		};

		r.block.base.header.parent_hash = parent.hash();
		r.block.base.header.number = parent.number + 1;
		r.block.base.header.author = author;
		r.block.base.header.set_timestamp_now(parent.timestamp());
		r.block.base.header.extra_data = extra_data;
		r.block.base.header.note_dirty();

		engine.populate_from_parent(&mut r.block.base.header, parent, gas_floor_target);
		engine.on_new_block(&mut r.block);
		r
	}

	/// Alter the author for the block.
	pub fn set_author(&mut self, author: Address) { self.block.base.header.set_author(author); }

	/// Alter the timestamp of the block.
	pub fn set_timestamp(&mut self, timestamp: u64) { self.block.base.header.set_timestamp(timestamp); }

	/// Alter the difficulty for the block.
	pub fn set_difficulty(&mut self, a: U256) { self.block.base.header.set_difficulty(a); }

	/// Alter the gas limit for the block.
	pub fn set_gas_limit(&mut self, a: U256) { self.block.base.header.set_gas_limit(a); }

	/// Alter the gas limit for the block.
	pub fn set_gas_used(&mut self, a: U256) { self.block.base.header.set_gas_used(a); }

	/// Alter the extra_data for the block.
	pub fn set_extra_data(&mut self, extra_data: Bytes) -> Result<(), BlockError> {
		if extra_data.len() > self.engine.maximum_extra_data_size() {
			Err(BlockError::ExtraDataOutOfBounds(OutOfBounds{min: None, max: Some(self.engine.maximum_extra_data_size()), found: extra_data.len()}))
		} else {
			self.block.base.header.set_extra_data(extra_data);
			Ok(())
		}
	}

	/// Add an uncle to the block, if possible.
	///
	/// NOTE Will check chain constraints and the uncle number but will NOT check
	/// that the header itself is actually valid.
	pub fn push_uncle(&mut self, valid_uncle_header: Header) -> Result<(), BlockError> {
		if self.block.base.uncles.len() + 1 > self.engine.maximum_uncle_count() {
			return Err(BlockError::TooManyUncles(OutOfBounds{min: None, max: Some(self.engine.maximum_uncle_count()), found: self.block.base.uncles.len() + 1}));
		}
		// TODO: check number
		// TODO: check not a direct ancestor (use last_hashes for that)
		self.block.base.uncles.push(valid_uncle_header);
		Ok(())
	}

	/// Get the environment info concerning this block.
	pub fn env_info(&self) -> EnvInfo {
		// TODO: memoise.
		EnvInfo {
			number: self.block.base.header.number,
			author: self.block.base.header.author.clone(),
			timestamp: self.block.base.header.timestamp,
			difficulty: self.block.base.header.difficulty.clone(),
			last_hashes: self.last_hashes.clone(),		// TODO: should be a reference.
			gas_used: self.block.receipts.last().map_or(U256::zero(), |r| r.gas_used),
			gas_limit: self.block.base.header.gas_limit.clone(),
		}
	}

	/// Push a transaction into the block.
	///
	/// If valid, it will be executed, and archived together with the receipt.
	pub fn push_transaction(&mut self, t: SignedTransaction, h: Option<H256>) -> Result<&Receipt, Error> {
		if self.block.transactions_set.contains(&t.hash()) {
			return Err(From::from(TransactionError::AlreadyImported));
		}

		let env_info = self.env_info();
//		info!("env_info says gas_used={}", env_info.gas_used);
		match self.block.state.apply(&env_info, self.engine, &t, self.block.traces.is_some()) {
			Ok(outcome) => {
				self.block.transactions_set.insert(h.unwrap_or_else(||t.hash()));
				self.block.base.transactions.push(t);
				let t = outcome.trace;
				self.block.traces.as_mut().map(|traces| traces.push(t.expect("self.block.traces.is_some(): so we must be tracing: qed")));
				self.block.receipts.push(outcome.receipt);
				Ok(&self.block.receipts.last().unwrap())
			}
			Err(x) => Err(From::from(x))
		}
	}

	/// Turn this into a `ClosedBlock`. A `BlockChain` must be provided in order to figure out the uncles.
	pub fn close(self) -> ClosedBlock {
		let mut s = self;

		let unclosed_state = s.block.state.clone();

		s.engine.on_close_block(&mut s.block);
		s.block.base.header.transactions_root = ordered_trie_root(s.block.base.transactions.iter().map(|ref e| e.rlp_bytes().to_vec()).collect());
		let uncle_bytes = s.block.base.uncles.iter().fold(RlpStream::new_list(s.block.base.uncles.len()), |mut s, u| {s.append_raw(&u.rlp(Seal::With), 1); s} ).out();
		s.block.base.header.uncles_hash = uncle_bytes.sha3();
		s.block.base.header.state_root = s.block.state.root().clone();
		s.block.base.header.receipts_root = ordered_trie_root(s.block.receipts.iter().map(|ref r| r.rlp_bytes().to_vec()).collect());
		s.block.base.header.log_bloom = s.block.receipts.iter().fold(LogBloom::zero(), |mut b, r| {b = &b | &r.log_bloom; b}); //TODO: use |= operator
		s.block.base.header.gas_used = s.block.receipts.last().map_or(U256::zero(), |r| r.gas_used);
		s.block.base.header.note_dirty();

		ClosedBlock {
			block: s.block,
			uncle_bytes: uncle_bytes,
			last_hashes: s.last_hashes,
			unclosed_state: unclosed_state,
		}
	}

	/// Turn this into a `LockedBlock`. A BlockChain must be provided in order to figure out the uncles.
	pub fn close_and_lock(self) -> LockedBlock {
		let mut s = self;

		s.engine.on_close_block(&mut s.block);
		s.block.base.header.transactions_root = ordered_trie_root(s.block.base.transactions.iter().map(|ref e| e.rlp_bytes().to_vec()).collect());
		let uncle_bytes = s.block.base.uncles.iter().fold(RlpStream::new_list(s.block.base.uncles.len()), |mut s, u| {s.append_raw(&u.rlp(Seal::With), 1); s} ).out();
		s.block.base.header.uncles_hash = uncle_bytes.sha3();
		s.block.base.header.state_root = s.block.state.root().clone();
		s.block.base.header.receipts_root = ordered_trie_root(s.block.receipts.iter().map(|ref r| r.rlp_bytes().to_vec()).collect());
		s.block.base.header.log_bloom = s.block.receipts.iter().fold(LogBloom::zero(), |mut b, r| {b = &b | &r.log_bloom; b}); //TODO: use |= operator
		s.block.base.header.gas_used = s.block.receipts.last().map_or(U256::zero(), |r| r.gas_used);
		s.block.base.header.note_dirty();

		LockedBlock {
			block: s.block,
			uncle_bytes: uncle_bytes,
			last_hashes: s.last_hashes,
		}
	}
}

impl<'x> IsBlock for OpenBlock<'x> {
	fn block(&self) -> &ExecutedBlock { &self.block }
}

impl<'x> IsBlock for ClosedBlock {
	fn block(&self) -> &ExecutedBlock { &self.block }
}

impl<'x> IsBlock for LockedBlock {
	fn block(&self) -> &ExecutedBlock { &self.block }
}

impl ClosedBlock {
	/// Get the hash of the header without seal arguments.
	pub fn hash(&self) -> H256 { self.header().rlp_sha3(Seal::Without) }

	/// Turn this into a `LockedBlock`, unable to be reopened again.
	pub fn lock(self) -> LockedBlock {
		LockedBlock {
			block: self.block,
			uncle_bytes: self.uncle_bytes,
			last_hashes: self.last_hashes,
		}
	}

	/// Given an engine reference, reopen the `ClosedBlock` into an `OpenBlock`.
	pub fn reopen(self, engine: &Engine) -> OpenBlock {
		// revert rewards (i.e. set state back at last transaction's state).
		let mut block = self.block;
		block.state = self.unclosed_state;
		OpenBlock {
			block: block,
			engine: engine,
			last_hashes: self.last_hashes,
		}
	}
}

impl LockedBlock {
	/// Get the hash of the header without seal arguments.
	pub fn hash(&self) -> H256 { self.header().rlp_sha3(Seal::Without) }

	/// Provide a valid seal in order to turn this into a `SealedBlock`.
	///
	/// NOTE: This does not check the validity of `seal` with the engine.
	pub fn seal(self, engine: &Engine, seal: Vec<Bytes>) -> Result<SealedBlock, BlockError> {
		let mut s = self;
		if seal.len() != engine.seal_fields() {
			return Err(BlockError::InvalidSealArity(Mismatch{expected: engine.seal_fields(), found: seal.len()}));
		}
		s.block.base.header.set_seal(seal);
		Ok(SealedBlock { block: s.block, uncle_bytes: s.uncle_bytes })
	}

	/// Provide a valid seal in order to turn this into a `SealedBlock`.
	/// This does check the validity of `seal` with the engine.
	/// Returns the `ClosedBlock` back again if the seal is no good.
	pub fn try_seal(self, engine: &Engine, seal: Vec<Bytes>) -> Result<SealedBlock, LockedBlock> {
		let mut s = self;
		s.block.base.header.set_seal(seal);
		match engine.verify_block_seal(&s.block.base.header) {
			Err(_) => Err(s),
			_ => Ok(SealedBlock { block: s.block, uncle_bytes: s.uncle_bytes }),
		}
	}

	/// Drop this object and return the underlieing database.
	pub fn drain(self) -> Box<JournalDB> { self.block.state.drop().1 }
}

impl SealedBlock {
	/// Get the RLP-encoding of the block.
	pub fn rlp_bytes(&self) -> Bytes {
		let mut block_rlp = RlpStream::new_list(3);
		self.block.base.header.stream_rlp(&mut block_rlp, Seal::With);
		block_rlp.append(&self.block.base.transactions);
		block_rlp.append_raw(&self.uncle_bytes, 1);
		block_rlp.out()
	}

	/// Drop this object and return the underlieing database.
	pub fn drain(self) -> Box<JournalDB> { self.block.state.drop().1 }
}

impl IsBlock for SealedBlock {
	fn block(&self) -> &ExecutedBlock { &self.block }
}

/// Enact the block given by block header, transactions and uncles
#[cfg_attr(feature="dev", allow(too_many_arguments))]
pub fn enact(header: &Header, transactions: &[SignedTransaction], uncles: &[Header], engine: &Engine, tracing: bool, db: Box<JournalDB>, parent: &Header, last_hashes: LastHashes) -> Result<LockedBlock, Error> {
	{
		if ::log::max_log_level() >= ::log::LogLevel::Trace {
			let s = State::from_existing(db.boxed_clone(), parent.state_root().clone(), engine.account_start_nonce());
			trace!("enact(): root={}, author={}, author_balance={}\n", s.root(), header.author(), s.balance(&header.author()));
		}
	}

	let mut b = OpenBlock::new(engine, tracing, db, parent, last_hashes, header.author().clone(), x!(3141562), header.extra_data().clone());
	b.set_difficulty(*header.difficulty());
	b.set_gas_limit(*header.gas_limit());
	b.set_timestamp(header.timestamp());
	for t in transactions { try!(b.push_transaction(t.clone(), None)); }
	for u in uncles { try!(b.push_uncle(u.clone())); }
	Ok(b.close_and_lock())
}

/// Enact the block given by `block_bytes` using `engine` on the database `db` with given `parent` block header
pub fn enact_bytes(block_bytes: &[u8], engine: &Engine, tracing: bool, db: Box<JournalDB>, parent: &Header, last_hashes: LastHashes) -> Result<LockedBlock, Error> {
	let block = BlockView::new(block_bytes);
	let header = block.header();
	enact(&header, &block.transactions(), &block.uncles(), engine, tracing, db, parent, last_hashes)
}

/// Enact the block given by `block_bytes` using `engine` on the database `db` with given `parent` block header
pub fn enact_verified(block: &PreverifiedBlock, engine: &Engine, tracing: bool, db: Box<JournalDB>, parent: &Header, last_hashes: LastHashes) -> Result<LockedBlock, Error> {
	let view = BlockView::new(&block.bytes);
	enact(&block.header, &block.transactions, &view.uncles(), engine, tracing, db, parent, last_hashes)
}

/// Enact the block given by `block_bytes` using `engine` on the database `db` with given `parent` block header. Seal the block aferwards
pub fn enact_and_seal(block_bytes: &[u8], engine: &Engine, tracing: bool, db: Box<JournalDB>, parent: &Header, last_hashes: LastHashes) -> Result<SealedBlock, Error> {
	let header = BlockView::new(block_bytes).header_view();
	Ok(try!(try!(enact_bytes(block_bytes, engine, tracing, db, parent, last_hashes)).seal(engine, header.seal())))
}

#[cfg(test)]
mod tests {
	use tests::helpers::*;
	use super::*;
	use common::*;

	#[test]
	fn open_block() {
		use spec::*;
		let spec = Spec::new_test();
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let last_hashes = vec![genesis_header.hash()];
		let b = OpenBlock::new(engine.deref(), false, db, &genesis_header, last_hashes, Address::zero(), x!(3141562), vec![]);
		let b = b.close_and_lock();
		let _ = b.seal(engine.deref(), vec![]);
	}

	#[test]
	fn enact_block() {
		use spec::*;
		let spec = Spec::new_test();
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();

		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let b = OpenBlock::new(engine.deref(), false, db, &genesis_header, vec![genesis_header.hash()], Address::zero(), x!(3141562), vec![]).close_and_lock().seal(engine.deref(), vec![]).unwrap();
		let orig_bytes = b.rlp_bytes();
		let orig_db = b.drain();

		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let e = enact_and_seal(&orig_bytes, engine.deref(), false, db, &genesis_header, vec![genesis_header.hash()]).unwrap();

		assert_eq!(e.rlp_bytes(), orig_bytes);

		let db = e.drain();
		assert_eq!(orig_db.keys(), db.keys());
		assert!(orig_db.keys().iter().filter(|k| orig_db.get(k.0) != db.get(k.0)).next() == None);
	}

	#[test]
	fn enact_block_with_uncle() {
		use spec::*;
		let spec = Spec::new_test();
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();

		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let mut open_block = OpenBlock::new(engine.deref(), false, db, &genesis_header, vec![genesis_header.hash()], Address::zero(), x!(3141562), vec![]);
		let mut uncle1_header = Header::new();
		uncle1_header.extra_data = b"uncle1".to_vec();
		let mut uncle2_header = Header::new();
		uncle2_header.extra_data = b"uncle2".to_vec();
		open_block.push_uncle(uncle1_header).unwrap();
		open_block.push_uncle(uncle2_header).unwrap();
		let b = open_block.close_and_lock().seal(engine.deref(), vec![]).unwrap();

		let orig_bytes = b.rlp_bytes();
		let orig_db = b.drain();

		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let e = enact_and_seal(&orig_bytes, engine.deref(), false, db, &genesis_header, vec![genesis_header.hash()]).unwrap();

		let bytes = e.rlp_bytes();
		assert_eq!(bytes, orig_bytes);
		let uncles = BlockView::new(&bytes).uncles();
		assert_eq!(uncles[1].extra_data, b"uncle2");

		let db = e.drain();
		assert_eq!(orig_db.keys(), db.keys());
		assert!(orig_db.keys().iter().filter(|k| orig_db.get(k.0) != db.get(k.0)).next() == None);
	}
}
