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

//! Base data structure of this module is `Block`.
//!
//! Blocks can be produced by a local node or they may be received from the network.
//!
//! To create a block locally, we start with an `OpenBlock`. This block is mutable
//! and can be appended to with transactions and uncles.
//!
//! When ready, `OpenBlock` can be closed and turned into a `ClosedBlock`. A `ClosedBlock` can
//! be reopend again by a miner under certain circumstances. On block close, state commit is
//! performed.
//!
//! `LockedBlock` is a version of a `ClosedBlock` that cannot be reopened. It can be sealed
//! using an engine.
//!
//! `ExecutedBlock` is an underlaying data structure used by all structs above to store block
//! related info.

use std::{cmp, ops};
use std::collections::HashSet;
use std::sync::Arc;

use bytes::Bytes;
use ethereum_types::{H256, U256, Address, Bloom};

use engines::EthEngine;
use error::{Error, BlockError};
use factory::Factories;
use state_db::StateDB;
use state::State;
use trace::Tracing;
use triehash::ordered_trie_root;
use unexpected::{Mismatch, OutOfBounds};
use verification::PreverifiedBlock;
use vm::{EnvInfo, LastHashes};

use hash::keccak;
use rlp::{RlpStream, Encodable, encode_list};
use types::transaction::{SignedTransaction, Error as TransactionError};
use types::header::{Header, ExtendedHeader};
use types::receipt::{Receipt, TransactionOutcome};

/// Block that is ready for transactions to be added.
///
/// It's a bit like a Vec<Transaction>, except that whenever a transaction is pushed, we execute it and
/// maintain the system `state()`. We also archive execution receipts in preparation for later block creation.
pub struct OpenBlock<'x> {
	block: ExecutedBlock,
	engine: &'x EthEngine,
}

/// Just like `OpenBlock`, except that we've applied `Engine::on_close_block`, finished up the non-seal header fields,
/// and collected the uncles.
///
/// There is no function available to push a transaction.
#[derive(Clone)]
pub struct ClosedBlock {
	block: ExecutedBlock,
	unclosed_state: State<StateDB>,
}

/// Just like `ClosedBlock` except that we can't reopen it and it's faster.
///
/// We actually store the post-`Engine::on_close_block` state, unlike in `ClosedBlock` where it's the pre.
#[derive(Clone)]
pub struct LockedBlock {
	block: ExecutedBlock,
}

/// A block that has a valid seal.
///
/// The block's header has valid seal arguments. The block cannot be reversed into a `ClosedBlock` or `OpenBlock`.
pub struct SealedBlock {
	block: ExecutedBlock,
}

/// An internal type for a block's common elements.
#[derive(Clone)]
pub struct ExecutedBlock {
	/// Executed block header.
	pub header: Header,
	/// Executed transactions.
	pub transactions: Vec<SignedTransaction>,
	/// Uncles.
	pub uncles: Vec<Header>,
	/// Transaction receipts.
	pub receipts: Vec<Receipt>,
	/// Hashes of already executed transactions.
	pub transactions_set: HashSet<H256>,
	/// Underlaying state.
	pub state: State<StateDB>,
	/// Transaction traces.
	pub traces: Tracing,
	/// Hashes of last 256 blocks.
	pub last_hashes: Arc<LastHashes>,
}

impl ExecutedBlock {
	/// Create a new block from the given `state`.
	fn new(state: State<StateDB>, last_hashes: Arc<LastHashes>, tracing: bool) -> ExecutedBlock {
		ExecutedBlock {
			header: Default::default(),
			transactions: Default::default(),
			uncles: Default::default(),
			receipts: Default::default(),
			transactions_set: Default::default(),
			state: state,
			traces: if tracing {
				Tracing::enabled()
			} else {
				Tracing::Disabled
			},
			last_hashes: last_hashes,
		}
	}

	/// Get the environment info concerning this block.
	pub fn env_info(&self) -> EnvInfo {
		// TODO: memoise.
		EnvInfo {
			number: self.header.number(),
			author: self.header.author().clone(),
			timestamp: self.header.timestamp(),
			difficulty: self.header.difficulty().clone(),
			last_hashes: self.last_hashes.clone(),
			gas_used: self.receipts.last().map_or(U256::zero(), |r| r.gas_used),
			gas_limit: self.header.gas_limit().clone(),
		}
	}

	/// Get mutable access to a state.
	pub fn state_mut(&mut self) -> &mut State<StateDB> {
		&mut self.state
	}

	/// Get mutable reference to traces.
	pub fn traces_mut(&mut self) -> &mut Tracing {
		&mut self.traces
	}
}

/// Trait for an object that owns an `ExecutedBlock`
pub trait Drain {
	/// Returns `ExecutedBlock`
	fn drain(self) -> ExecutedBlock;
}

impl<'x> OpenBlock<'x> {
	/// Create a new `OpenBlock` ready for transaction pushing.
	pub fn new<'a, I: IntoIterator<Item = ExtendedHeader>>(
		engine: &'x EthEngine,
		factories: Factories,
		tracing: bool,
		db: StateDB,
		parent: &Header,
		last_hashes: Arc<LastHashes>,
		author: Address,
		gas_range_target: (U256, U256),
		extra_data: Bytes,
		is_epoch_begin: bool,
		ancestry: I,
	) -> Result<Self, Error> {
		let number = parent.number() + 1;
		let state = State::from_existing(db, parent.state_root().clone(), engine.account_start_nonce(number), factories)?;
		let mut r = OpenBlock {
			block: ExecutedBlock::new(state, last_hashes, tracing),
			engine: engine,
		};

		r.block.header.set_parent_hash(parent.hash());
		r.block.header.set_number(number);
		r.block.header.set_author(author);
		r.block.header.set_timestamp(engine.open_block_header_timestamp(parent.timestamp()));
		r.block.header.set_extra_data(extra_data);

		let gas_floor_target = cmp::max(gas_range_target.0, engine.params().min_gas_limit);
		let gas_ceil_target = cmp::max(gas_range_target.1, gas_floor_target);

		engine.machine().populate_from_parent(&mut r.block.header, parent, gas_floor_target, gas_ceil_target);
		engine.populate_from_parent(&mut r.block.header, parent);

		engine.machine().on_new_block(&mut r.block)?;
		engine.on_new_block(&mut r.block, is_epoch_begin, &mut ancestry.into_iter())?;

		Ok(r)
	}

	/// Alter the timestamp of the block.
	pub fn set_timestamp(&mut self, timestamp: u64) {
		self.block.header.set_timestamp(timestamp);
	}

	/// Removes block gas limit.
	pub fn remove_gas_limit(&mut self) {
		self.block.header.set_gas_limit(U256::max_value());
	}

	/// Add an uncle to the block, if possible.
	///
	/// NOTE Will check chain constraints and the uncle number but will NOT check
	/// that the header itself is actually valid.
	pub fn push_uncle(&mut self, valid_uncle_header: Header) -> Result<(), BlockError> {
		let max_uncles = self.engine.maximum_uncle_count(self.block.header.number());
		if self.block.uncles.len() + 1 > max_uncles {
			return Err(BlockError::TooManyUncles(OutOfBounds{
				min: None,
				max: Some(max_uncles),
				found: self.block.uncles.len() + 1,
			}));
		}
		// TODO: check number
		// TODO: check not a direct ancestor (use last_hashes for that)
		self.block.uncles.push(valid_uncle_header);
		Ok(())
	}

	/// Push a transaction into the block.
	///
	/// If valid, it will be executed, and archived together with the receipt.
	pub fn push_transaction(&mut self, t: SignedTransaction, h: Option<H256>) -> Result<&Receipt, Error> {
		if self.block.transactions_set.contains(&t.hash()) {
			return Err(TransactionError::AlreadyImported.into());
		}

		let env_info = self.block.env_info();
		let outcome = self.block.state.apply(&env_info, self.engine.machine(), &t, self.block.traces.is_enabled())?;

		self.block.transactions_set.insert(h.unwrap_or_else(||t.hash()));
		self.block.transactions.push(t.into());
		if let Tracing::Enabled(ref mut traces) = self.block.traces {
			traces.push(outcome.trace.into());
		}
		self.block.receipts.push(outcome.receipt);
		Ok(self.block.receipts.last().expect("receipt just pushed; qed"))
	}

	/// Push transactions onto the block.
	#[cfg(not(feature = "slow-blocks"))]
	fn push_transactions(&mut self, transactions: Vec<SignedTransaction>) -> Result<(), Error> {
		for t in transactions {
			self.push_transaction(t, None)?;
		}
		Ok(())
	}

	/// Push transactions onto the block.
	#[cfg(feature = "slow-blocks")]
	fn push_transactions(&mut self, transactions: Vec<SignedTransaction>) -> Result<(), Error> {
		use std::time;

		let slow_tx = option_env!("SLOW_TX_DURATION").and_then(|v| v.parse().ok()).unwrap_or(100);
		for t in transactions {
			let hash = t.hash();
			let start = time::Instant::now();
			self.push_transaction(t, None)?;
			let took = start.elapsed();
			let took_ms = took.as_secs() * 1000 + took.subsec_nanos() as u64 / 1000000;
			if took > time::Duration::from_millis(slow_tx) {
				warn!("Heavy ({} ms) transaction in block {:?}: {:?}", took_ms, self.block.header().number(), hash);
			}
			debug!(target: "tx", "Transaction {:?} took: {} ms", hash, took_ms);
		}

		Ok(())
	}

	/// Populate self from a header.
	fn populate_from(&mut self, header: &Header) {
		self.block.header.set_difficulty(*header.difficulty());
		self.block.header.set_gas_limit(*header.gas_limit());
		self.block.header.set_timestamp(header.timestamp());
		self.block.header.set_uncles_hash(*header.uncles_hash());
		self.block.header.set_transactions_root(*header.transactions_root());
		// TODO: that's horrible. set only for backwards compatibility
		if header.extra_data().len() > self.engine.maximum_extra_data_size() {
			warn!("Couldn't set extradata. Ignoring.");
		} else {
			self.block.header.set_extra_data(header.extra_data().clone());
		}
	}

	/// Turn this into a `ClosedBlock`.
	pub fn close(self) -> Result<ClosedBlock, Error> {
		let unclosed_state = self.block.state.clone();
		let locked = self.close_and_lock()?;

		Ok(ClosedBlock {
			block: locked.block,
			unclosed_state,
		})
	}

	/// Turn this into a `LockedBlock`.
	pub fn close_and_lock(self) -> Result<LockedBlock, Error> {
		let mut s = self;

		s.engine.on_close_block(&mut s.block)?;
		s.block.state.commit()?;

		s.block.header.set_transactions_root(ordered_trie_root(s.block.transactions.iter().map(|e| e.rlp_bytes())));
		let uncle_bytes = encode_list(&s.block.uncles);
		s.block.header.set_uncles_hash(keccak(&uncle_bytes));
		s.block.header.set_state_root(s.block.state.root().clone());
		s.block.header.set_receipts_root(ordered_trie_root(s.block.receipts.iter().map(|r| r.rlp_bytes())));
		s.block.header.set_log_bloom(s.block.receipts.iter().fold(Bloom::zero(), |mut b, r| {
			b.accrue_bloom(&r.log_bloom);
			b
		}));
		s.block.header.set_gas_used(s.block.receipts.last().map_or_else(U256::zero, |r| r.gas_used));

		Ok(LockedBlock {
			block: s.block,
		})
	}

	#[cfg(test)]
	/// Return mutable block reference. To be used in tests only.
	pub fn block_mut(&mut self) -> &mut ExecutedBlock { &mut self.block }
}

impl<'a> ops::Deref for OpenBlock<'a> {
	type Target = ExecutedBlock;

	fn deref(&self) -> &Self::Target {
		&self.block
	}
}

impl ops::Deref for ClosedBlock {
	type Target = ExecutedBlock;

	fn deref(&self) -> &Self::Target {
		&self.block
	}
}

impl ops::Deref for LockedBlock {
	type Target = ExecutedBlock;

	fn deref(&self) -> &Self::Target {
		&self.block
	}
}

impl ops::Deref for SealedBlock {
	type Target = ExecutedBlock;

	fn deref(&self) -> &Self::Target {
		&self.block
	}
}

impl ClosedBlock {
	/// Turn this into a `LockedBlock`, unable to be reopened again.
	pub fn lock(self) -> LockedBlock {
		LockedBlock {
			block: self.block,
		}
	}

	/// Given an engine reference, reopen the `ClosedBlock` into an `OpenBlock`.
	pub fn reopen(self, engine: &EthEngine) -> OpenBlock {
		// revert rewards (i.e. set state back at last transaction's state).
		let mut block = self.block;
		block.state = self.unclosed_state;
		OpenBlock {
			block: block,
			engine: engine,
		}
	}
}

impl LockedBlock {
	/// Removes outcomes from receipts and updates the receipt root.
	///
	/// This is done after the block is enacted for historical reasons.
	/// We allow inconsistency in receipts for some chains if `validate_receipts_transition`
	/// is set to non-zero value, so the check only happens if we detect
	/// unmatching root first and then fall back to striped receipts.
	pub fn strip_receipts_outcomes(&mut self) {
		for receipt in &mut self.block.receipts {
			receipt.outcome = TransactionOutcome::Unknown;
		}
		self.block.header.set_receipts_root(
			ordered_trie_root(self.block.receipts.iter().map(|r| r.rlp_bytes()))
		);
	}

	/// Provide a valid seal in order to turn this into a `SealedBlock`.
	///
	/// NOTE: This does not check the validity of `seal` with the engine.
	pub fn seal(self, engine: &EthEngine, seal: Vec<Bytes>) -> Result<SealedBlock, Error> {
		let expected_seal_fields = engine.seal_fields(&self.header);
		let mut s = self;
		if seal.len() != expected_seal_fields {
			Err(BlockError::InvalidSealArity(Mismatch {
				expected: expected_seal_fields,
				found: seal.len()
			}))?;
		}

		s.block.header.set_seal(seal);
		engine.on_seal_block(&mut s.block)?;
		s.block.header.compute_hash();

		Ok(SealedBlock {
			block: s.block
		})
	}

	/// Provide a valid seal in order to turn this into a `SealedBlock`.
	/// This does check the validity of `seal` with the engine.
	/// Returns the `ClosedBlock` back again if the seal is no good.
	/// TODO(https://github.com/paritytech/parity-ethereum/issues/10407): This is currently only used in POW chain call paths, we should really merge it with seal() above.
	pub fn try_seal(
		self,
		engine: &EthEngine,
		seal: Vec<Bytes>,
	) -> Result<SealedBlock, Error> {
		let mut s = self;
		s.block.header.set_seal(seal);
		s.block.header.compute_hash();

		// TODO: passing state context to avoid engines owning it?
		engine.verify_local_seal(&s.block.header)?;
		Ok(SealedBlock {
			block: s.block
		})
	}
}

impl Drain for LockedBlock {
	fn drain(self) -> ExecutedBlock {
		self.block
	}
}

impl SealedBlock {
	/// Get the RLP-encoding of the block.
	pub fn rlp_bytes(&self) -> Bytes {
		let mut block_rlp = RlpStream::new_list(3);
		block_rlp.append(&self.block.header);
		block_rlp.append_list(&self.block.transactions);
		block_rlp.append_list(&self.block.uncles);
		block_rlp.out()
	}
}

impl Drain for SealedBlock {
	fn drain(self) -> ExecutedBlock {
		self.block
	}
}

/// Enact the block given by block header, transactions and uncles
pub(crate) fn enact(
	header: Header,
	transactions: Vec<SignedTransaction>,
	uncles: Vec<Header>,
	engine: &EthEngine,
	tracing: bool,
	db: StateDB,
	parent: &Header,
	last_hashes: Arc<LastHashes>,
	factories: Factories,
	is_epoch_begin: bool,
	ancestry: &mut Iterator<Item=ExtendedHeader>,
) -> Result<LockedBlock, Error> {
	// For trace log
	let trace_state = if log_enabled!(target: "enact", ::log::Level::Trace) {
		Some(State::from_existing(db.boxed_clone(), parent.state_root().clone(), engine.account_start_nonce(parent.number() + 1), factories.clone())?)
	} else {
		None
	};

	let mut b = OpenBlock::new(
		engine,
		factories,
		tracing,
		db,
		parent,
		last_hashes,
		// Engine such as Clique will calculate author from extra_data.
		// this is only important for executing contracts as the 'executive_author'.
		engine.executive_author(&header)?,
		(3141562.into(), 31415620.into()),
		vec![],
		is_epoch_begin,
		ancestry,
	)?;

	if let Some(ref s) = trace_state {
		let env = b.env_info();
		let root = s.root();
		let author_balance = s.balance(&env.author)?;
		trace!(target: "enact", "num={}, root={}, author={}, author_balance={}\n",
				b.block.header.number(), root, env.author, author_balance);
	}

	b.populate_from(&header);
	b.push_transactions(transactions)?;

	for u in uncles {
		b.push_uncle(u)?;
	}

	b.close_and_lock()
}

/// Enact the block given by `block_bytes` using `engine` on the database `db` with given `parent` block header
pub fn enact_verified(
	block: PreverifiedBlock,
	engine: &EthEngine,
	tracing: bool,
	db: StateDB,
	parent: &Header,
	last_hashes: Arc<LastHashes>,
	factories: Factories,
	is_epoch_begin: bool,
	ancestry: &mut Iterator<Item=ExtendedHeader>,
) -> Result<LockedBlock, Error> {

	enact(
		block.header,
		block.transactions,
		block.uncles,
		engine,
		tracing,
		db,
		parent,
		last_hashes,
		factories,
		is_epoch_begin,
		ancestry,
	)
}

#[cfg(test)]
mod tests {
	use test_helpers::get_temp_state_db;
	use super::*;
	use engines::EthEngine;
	use vm::LastHashes;
	use error::Error;
	use factory::Factories;
	use state_db::StateDB;
	use ethereum_types::Address;
	use std::sync::Arc;
	use verification::queue::kind::blocks::Unverified;
	use types::transaction::SignedTransaction;
	use types::header::Header;
	use types::view;
	use types::views::BlockView;

	/// Enact the block given by `block_bytes` using `engine` on the database `db` with given `parent` block header
	fn enact_bytes(
		block_bytes: Vec<u8>,
		engine: &EthEngine,
		tracing: bool,
		db: StateDB,
		parent: &Header,
		last_hashes: Arc<LastHashes>,
		factories: Factories,
	) -> Result<LockedBlock, Error> {

		let block = Unverified::from_rlp(block_bytes)?;
		let header = block.header;
		let transactions: Result<Vec<_>, Error> = block
			.transactions
			.into_iter()
			.map(SignedTransaction::new)
			.map(|r| r.map_err(Into::into))
			.collect();
		let transactions = transactions?;

		{
			if ::log::max_level() >= ::log::Level::Trace {
				let s = State::from_existing(db.boxed_clone(), parent.state_root().clone(), engine.account_start_nonce(parent.number() + 1), factories.clone())?;
				trace!(target: "enact", "num={}, root={}, author={}, author_balance={}\n",
					header.number(), s.root(), header.author(), s.balance(&header.author())?);
			}
		}

		let mut b = OpenBlock::new(
			engine,
			factories,
			tracing,
			db,
			parent,
			last_hashes,
			Address::new(),
			(3141562.into(), 31415620.into()),
			vec![],
			false,
			None,
		)?;

		b.populate_from(&header);
		b.push_transactions(transactions)?;

		for u in block.uncles {
			b.push_uncle(u)?;
		}

		b.close_and_lock()
	}

	/// Enact the block given by `block_bytes` using `engine` on the database `db` with given `parent` block header. Seal the block aferwards
	fn enact_and_seal(
		block_bytes: Vec<u8>,
		engine: &EthEngine,
		tracing: bool,
		db: StateDB,
		parent: &Header,
		last_hashes: Arc<LastHashes>,
		factories: Factories,
	) -> Result<SealedBlock, Error> {
		let header = Unverified::from_rlp(block_bytes.clone())?.header;
		Ok(enact_bytes(block_bytes, engine, tracing, db, parent, last_hashes, factories)?
			.seal(engine, header.seal().to_vec())?)
	}

	#[test]
	fn open_block() {
		use spec::*;
		let spec = Spec::new_test();
		let genesis_header = spec.genesis_header();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(&*spec.engine, Default::default(), false, db, &genesis_header, last_hashes, Address::zero(), (3141562.into(), 31415620.into()), vec![], false, None).unwrap();
		let b = b.close_and_lock().unwrap();
		let _ = b.seal(&*spec.engine, vec![]);
	}

	#[test]
	fn enact_block() {
		use spec::*;
		let spec = Spec::new_test();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();

		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes.clone(), Address::zero(), (3141562.into(), 31415620.into()), vec![], false, None).unwrap()
			.close_and_lock().unwrap().seal(engine, vec![]).unwrap();
		let orig_bytes = b.rlp_bytes();
		let orig_db = b.drain().state.drop().1;

		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let e = enact_and_seal(orig_bytes.clone(), engine, false, db, &genesis_header, last_hashes, Default::default()).unwrap();

		assert_eq!(e.rlp_bytes(), orig_bytes);

		let db = e.drain().state.drop().1;
		assert_eq!(orig_db.journal_db().keys(), db.journal_db().keys());
		assert!(orig_db.journal_db().keys().iter().filter(|k| orig_db.journal_db().get(k.0) != db.journal_db().get(k.0)).next() == None);
	}

	#[test]
	fn enact_block_with_uncle() {
		use spec::*;
		let spec = Spec::new_test();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();

		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let mut open_block = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes.clone(), Address::zero(), (3141562.into(), 31415620.into()), vec![], false, None).unwrap();
		let mut uncle1_header = Header::new();
		uncle1_header.set_extra_data(b"uncle1".to_vec());
		let mut uncle2_header = Header::new();
		uncle2_header.set_extra_data(b"uncle2".to_vec());
		open_block.push_uncle(uncle1_header).unwrap();
		open_block.push_uncle(uncle2_header).unwrap();
		let b = open_block.close_and_lock().unwrap().seal(engine, vec![]).unwrap();

		let orig_bytes = b.rlp_bytes();
		let orig_db = b.drain().state.drop().1;

		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let e = enact_and_seal(orig_bytes.clone(), engine, false, db, &genesis_header, last_hashes, Default::default()).unwrap();

		let bytes = e.rlp_bytes();
		assert_eq!(bytes, orig_bytes);
		let uncles = view!(BlockView, &bytes).uncles();
		assert_eq!(uncles[1].extra_data(), b"uncle2");

		let db = e.drain().state.drop().1;
		assert_eq!(orig_db.journal_db().keys(), db.journal_db().keys());
		assert!(orig_db.journal_db().keys().iter().filter(|k| orig_db.journal_db().get(k.0) != db.journal_db().get(k.0)).next() == None);
	}
}
