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

//! Block and transaction verification functions
//!
//! Block verification is done in 3 steps
//! 1. Quick verification upon adding to the block queue
//! 2. Signatures verification done in the queue.
//! 3. Final verification against the blockchain done before enactment.

use std::collections::HashSet;

use blockchain::*;
use client::BlockChainClient;
use engines::EthEngine;
use error::{BlockError, Error};
use header::{BlockNumber, Header};
use transaction::SignedTransaction;
use views::BlockView;

use ethereum_types::{H256, U256};
use bytes::Bytes;
use hash::keccak;
use heapsize::HeapSizeOf;
use rlp::UntrustedRlp;
use time::get_time;
use triehash::ordered_trie_root;
use unexpected::{Mismatch, OutOfBounds};

/// Preprocessed block data gathered in `verify_block_unordered` call
pub struct PreverifiedBlock {
	/// Populated block header
	pub header: Header,
	/// Populated block transactions
	pub transactions: Vec<SignedTransaction>,
	/// Block bytes
	pub bytes: Bytes,
}

impl HeapSizeOf for PreverifiedBlock {
	fn heap_size_of_children(&self) -> usize {
		self.header.heap_size_of_children()
			+ self.transactions.heap_size_of_children()
			+ self.bytes.heap_size_of_children()
	}
}

/// Phase 1 quick block verification. Only does checks that are cheap. Operates on a single block
pub fn verify_block_basic(header: &Header, bytes: &[u8], engine: &EthEngine) -> Result<(), Error> {
	verify_header_params(&header, engine, true)?;
	verify_block_integrity(bytes, &header.transactions_root(), &header.uncles_hash())?;
	engine.verify_block_basic(&header)?;
	for u in UntrustedRlp::new(bytes).at(2)?.iter().map(|rlp| rlp.as_val::<Header>()) {
		let u = u?;
		verify_header_params(&u, engine, false)?;
		engine.verify_block_basic(&u)?;
	}
	// Verify transactions.
	// TODO: either use transaction views or cache the decoded transactions.
	let v = BlockView::new(bytes);
	for t in v.transactions() {
		engine.verify_transaction_basic(&t, &header)?;
	}
	Ok(())
}

/// Phase 2 verification. Perform costly checks such as transaction signatures and block nonce for ethash.
/// Still operates on a individual block
/// Returns a `PreverifiedBlock` structure populated with transactions
pub fn verify_block_unordered(header: Header, bytes: Bytes, engine: &EthEngine, check_seal: bool) -> Result<PreverifiedBlock, Error> {
	if check_seal {
		engine.verify_block_unordered(&header)?;
		for u in UntrustedRlp::new(&bytes).at(2)?.iter().map(|rlp| rlp.as_val::<Header>()) {
			engine.verify_block_unordered(&u?)?;
		}
	}
	// Verify transactions.
	let mut transactions = Vec::new();
	let nonce_cap = if header.number() >= engine.params().dust_protection_transition {
		Some((engine.params().nonce_cap_increment * header.number()).into())
	} else { None };
	{
		let v = BlockView::new(&bytes);
		for t in v.transactions() {
			let t = engine.verify_transaction_unordered(t, &header)?;
			if let Some(max_nonce) = nonce_cap {
				if t.nonce >= max_nonce {
					return Err(BlockError::TooManyTransactions(t.sender()).into());
				}
			}
			transactions.push(t);
		}
	}
	Ok(PreverifiedBlock {
		header: header,
		transactions: transactions,
		bytes: bytes,
	})
}

/// Parameters for full verification of block family: block bytes, transactions, blockchain, and state access.
pub type FullFamilyParams<'a> = (&'a [u8], &'a [SignedTransaction], &'a BlockProvider, &'a BlockChainClient);

/// Phase 3 verification. Check block information against parent and uncles.
pub fn verify_block_family(header: &Header, parent: &Header, engine: &EthEngine, do_full: Option<FullFamilyParams>) -> Result<(), Error> {
	// TODO: verify timestamp
	verify_parent(&header, &parent, engine.params().gas_limit_bound_divisor)?;
	engine.verify_block_family(&header, &parent)?;

	let (bytes, txs, bc, client) = match do_full {
		Some(x) => x,
		None => return Ok(()),
	};

	verify_uncles(header, bytes, bc, engine)?;

	for transaction in txs {
		engine.machine().verify_transaction(transaction, header, client)?;
	}

	Ok(())
}

fn verify_uncles(header: &Header, bytes: &[u8], bc: &BlockProvider, engine: &EthEngine) -> Result<(), Error> {
	let num_uncles = UntrustedRlp::new(bytes).at(2)?.item_count()?;
	let max_uncles = engine.maximum_uncle_count(header.number());
	if num_uncles != 0 {
		if num_uncles > max_uncles {
			return Err(From::from(BlockError::TooManyUncles(OutOfBounds {
				min: None,
				max: Some(max_uncles),
				found: num_uncles,
			})));
		}

		let mut excluded = HashSet::new();
		excluded.insert(header.hash());
		let mut hash = header.parent_hash().clone();
		excluded.insert(hash.clone());
		for _ in 0..engine.maximum_uncle_age() {
			match bc.block_details(&hash) {
				Some(details) => {
					excluded.insert(details.parent.clone());
					let b = bc.block(&hash)
						.expect("parent already known to be stored; qed");
					excluded.extend(b.uncle_hashes());
					hash = details.parent;
				}
				None => break
			}
		}

		let mut verified = HashSet::new();
		for uncle in UntrustedRlp::new(bytes).at(2)?.iter().map(|rlp| rlp.as_val::<Header>()) {
			let uncle = uncle?;
			if excluded.contains(&uncle.hash()) {
				return Err(From::from(BlockError::UncleInChain(uncle.hash())))
			}

			if verified.contains(&uncle.hash()) {
				return Err(From::from(BlockError::DuplicateUncle(uncle.hash())))
			}

			// m_currentBlock.number() - uncle.number()		m_cB.n - uP.n()
			// 1											2
			// 2
			// 3
			// 4
			// 5
			// 6											7
			//												(8 Invalid)

			let depth = if header.number() > uncle.number() { header.number() - uncle.number() } else { 0 };
			if depth > engine.maximum_uncle_age() as u64 {
				return Err(From::from(BlockError::UncleTooOld(OutOfBounds { min: Some(header.number() - depth), max: Some(header.number() - 1), found: uncle.number() })));
			}
			else if depth < 1 {
				return Err(From::from(BlockError::UncleIsBrother(OutOfBounds { min: Some(header.number() - depth), max: Some(header.number() - 1), found: uncle.number() })));
			}

			// cB
			// cB.p^1	    1 depth, valid uncle
			// cB.p^2	---/  2
			// cB.p^3	-----/  3
			// cB.p^4	-------/  4
			// cB.p^5	---------/  5
			// cB.p^6	-----------/  6
			// cB.p^7	-------------/
			// cB.p^8
			let mut expected_uncle_parent = header.parent_hash().clone();
			let uncle_parent = bc.block_header(&uncle.parent_hash()).ok_or_else(|| Error::from(BlockError::UnknownUncleParent(uncle.parent_hash().clone())))?;
			for _ in 0..depth {
				match bc.block_details(&expected_uncle_parent) {
					Some(details) => {
						expected_uncle_parent = details.parent;
					},
					None => break
				}
			}
			if expected_uncle_parent != uncle_parent.hash() {
				return Err(From::from(BlockError::UncleParentNotInChain(uncle_parent.hash())));
			}

			verify_parent(&uncle, &uncle_parent, engine.params().gas_limit_bound_divisor)?;
			engine.verify_block_family(&uncle, &uncle_parent)?;
			verified.insert(uncle.hash());
		}
	}

	Ok(())
}

/// Phase 4 verification. Check block information against transaction enactment results,
pub fn verify_block_final(expected: &Header, got: &Header) -> Result<(), Error> {
	if expected.gas_used() != got.gas_used() {
		return Err(From::from(BlockError::InvalidGasUsed(Mismatch { expected: expected.gas_used().clone(), found: got.gas_used().clone() })))
	}
	if expected.log_bloom() != got.log_bloom() {
		return Err(From::from(BlockError::InvalidLogBloom(Mismatch { expected: expected.log_bloom().clone(), found: got.log_bloom().clone() })))
	}
	if expected.state_root() != got.state_root() {
		return Err(From::from(BlockError::InvalidStateRoot(Mismatch { expected: expected.state_root().clone(), found: got.state_root().clone() })))
	}
	if expected.receipts_root() != got.receipts_root() {
		return Err(From::from(BlockError::InvalidReceiptsRoot(Mismatch { expected: expected.receipts_root().clone(), found: got.receipts_root().clone() })))
	}
	Ok(())
}

/// Check basic header parameters.
pub fn verify_header_params(header: &Header, engine: &EthEngine, is_full: bool) -> Result<(), Error> {
	if header.seal().len() != engine.seal_fields() {
		return Err(From::from(BlockError::InvalidSealArity(
			Mismatch { expected: engine.seal_fields(), found: header.seal().len() }
		)));
	}

	if header.number() >= From::from(BlockNumber::max_value()) {
		return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { max: Some(From::from(BlockNumber::max_value())), min: None, found: header.number() })))
	}
	if header.gas_used() > header.gas_limit() {
		return Err(From::from(BlockError::TooMuchGasUsed(OutOfBounds { max: Some(header.gas_limit().clone()), min: None, found: header.gas_used().clone() })));
	}
	let min_gas_limit = engine.params().min_gas_limit;
	if header.gas_limit() < &min_gas_limit {
		return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas_limit), max: None, found: header.gas_limit().clone() })));
	}
	let maximum_extra_data_size = engine.maximum_extra_data_size();
	if header.number() != 0 && header.extra_data().len() > maximum_extra_data_size {
		return Err(From::from(BlockError::ExtraDataOutOfBounds(OutOfBounds { min: None, max: Some(maximum_extra_data_size), found: header.extra_data().len() })));
	}

	if let Some(ref ext) = engine.machine().ethash_extensions() {
		if header.number() >= ext.dao_hardfork_transition &&
			header.number() <= ext.dao_hardfork_transition + 9 &&
			header.extra_data()[..] != b"dao-hard-fork"[..] {
			return Err(From::from(BlockError::ExtraDataOutOfBounds(OutOfBounds { min: None, max: None, found: 0 })));
		}
	}

	if is_full {
		const ACCEPTABLE_DRIFT_SECS: u64 = 15;
		let max_time = get_time().sec as u64 + ACCEPTABLE_DRIFT_SECS;
		let invalid_threshold = max_time + ACCEPTABLE_DRIFT_SECS * 9;
		let timestamp = header.timestamp();

		if timestamp > invalid_threshold {
			return Err(From::from(BlockError::InvalidTimestamp(OutOfBounds { max: Some(max_time), min: None, found: timestamp })))
		}

		if timestamp > max_time {
			return Err(From::from(BlockError::TemporarilyInvalid(OutOfBounds { max: Some(max_time), min: None, found: timestamp })))
		}
	}

	Ok(())
}

/// Check header parameters agains parent header.
fn verify_parent(header: &Header, parent: &Header, gas_limit_divisor: U256) -> Result<(), Error> {
	if !header.parent_hash().is_zero() && &parent.hash() != header.parent_hash() {
		return Err(From::from(BlockError::InvalidParentHash(Mismatch { expected: parent.hash(), found: header.parent_hash().clone() })))
	}
	if header.timestamp() <= parent.timestamp() {
		return Err(From::from(BlockError::InvalidTimestamp(OutOfBounds { max: None, min: Some(parent.timestamp() + 1), found: header.timestamp() })))
	}
	if header.number() != parent.number() + 1 {
		return Err(From::from(BlockError::InvalidNumber(Mismatch { expected: parent.number() + 1, found: header.number() })));
	}

	if header.number() == 0 {
		return Err(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() }).into());
	}

	let parent_gas_limit = *parent.gas_limit();
	let min_gas = parent_gas_limit - parent_gas_limit / gas_limit_divisor;
	let max_gas = parent_gas_limit + parent_gas_limit / gas_limit_divisor;
	if header.gas_limit() <= &min_gas || header.gas_limit() >= &max_gas {
		return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit().clone() })));
	}

	Ok(())
}

/// Verify block data against header: transactions root and uncles hash.
fn verify_block_integrity(block: &[u8], transactions_root: &H256, uncles_hash: &H256) -> Result<(), Error> {
	let block = UntrustedRlp::new(block);
	let tx = block.at(1)?;
	let expected_root = &ordered_trie_root(tx.iter().map(|r| r.as_raw().to_vec())); //TODO: get rid of vectors here
	if expected_root != transactions_root {
		return Err(From::from(BlockError::InvalidTransactionsRoot(Mismatch { expected: expected_root.clone(), found: transactions_root.clone() })))
	}
	let expected_uncles = &keccak(block.at(2)?.as_raw());
	if expected_uncles != uncles_hash {
		return Err(From::from(BlockError::InvalidUnclesHash(Mismatch { expected: expected_uncles.clone(), found: uncles_hash.clone() })))
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::collections::{BTreeMap, HashMap};
	use ethereum_types::{H256, Bloom, U256};
	use blockchain::extras::{BlockDetails, TransactionAddress, BlockReceipts};
	use encoded;
	use hash::keccak;
	use engines::EthEngine;
	use error::BlockError::*;
	use ethkey::{Random, Generator};
	use spec::{CommonParams, Spec};
	use tests::helpers::{create_test_block_with_data, create_test_block};
	use time::get_time;
	use transaction::{SignedTransaction, Transaction, UnverifiedTransaction, Action};
	use types::log_entry::{LogEntry, LocalizedLogEntry};

	fn check_ok(result: Result<(), Error>) {
		result.unwrap_or_else(|e| panic!("Block verification failed: {:?}", e));
	}

	fn check_fail(result: Result<(), Error>, e: BlockError) {
		match result {
			Err(Error::Block(ref error)) if *error == e => (),
			Err(other) => panic!("Block verification failed.\nExpected: {:?}\nGot: {:?}", e, other),
			Ok(_) => panic!("Block verification failed.\nExpected: {:?}\nGot: Ok", e),
		}
	}

	fn check_fail_timestamp(result: Result<(), Error>, temp: bool) {
		let name = if temp { "TemporarilyInvalid" } else { "InvalidTimestamp" };
		match result {
			Err(Error::Block(BlockError::InvalidTimestamp(_))) if !temp => (),
			Err(Error::Block(BlockError::TemporarilyInvalid(_))) if temp => (),
			Err(other) => panic!("Block verification failed.\nExpected: {}\nGot: {:?}", name, other),
			Ok(_) => panic!("Block verification failed.\nExpected: {}\nGot: Ok", name),
		}
	}

	struct TestBlockChain {
		blocks: HashMap<H256, Bytes>,
		numbers: HashMap<BlockNumber, H256>,
	}

	impl Default for TestBlockChain {
		fn default() -> Self {
			TestBlockChain::new()
		}
	}

	impl TestBlockChain {
		pub fn new() -> Self {
			TestBlockChain {
				blocks: HashMap::new(),
				numbers: HashMap::new(),
			}
		}

		pub fn insert(&mut self, bytes: Bytes) {
			let number = BlockView::new(&bytes).header_view().number();
			let hash = BlockView::new(&bytes).header_view().hash();
			self.blocks.insert(hash.clone(), bytes);
			self.numbers.insert(number, hash.clone());
		}
	}

	impl BlockProvider for TestBlockChain {
		fn is_known(&self, hash: &H256) -> bool {
			self.blocks.contains_key(hash)
		}

		fn first_block(&self) -> Option<H256> {
			unimplemented!()
		}

		/// Get raw block data
		fn block(&self, hash: &H256) -> Option<encoded::Block> {
			self.blocks.get(hash).cloned().map(encoded::Block::new)
		}

		fn block_header_data(&self, hash: &H256) -> Option<encoded::Header> {
			self.block(hash)
				.map(|b| b.header_view().rlp().as_raw().to_vec())
				.map(encoded::Header::new)
		}

		fn block_body(&self, hash: &H256) -> Option<encoded::Body> {
			self.block(hash)
				.map(|b| BlockChain::block_to_body(&b.into_inner()))
				.map(encoded::Body::new)
		}

		fn best_ancient_block(&self) -> Option<H256> {
			None
		}

		/// Get the familial details concerning a block.
		fn block_details(&self, hash: &H256) -> Option<BlockDetails> {
			self.blocks.get(hash).map(|bytes| {
				let header = BlockView::new(bytes).header();
				BlockDetails {
					number: header.number(),
					total_difficulty: header.difficulty().clone(),
					parent: header.parent_hash().clone(),
					children: Vec::new(),
				}
			})
		}

		fn transaction_address(&self, _hash: &H256) -> Option<TransactionAddress> {
			unimplemented!()
		}

		/// Get the hash of given block's number.
		fn block_hash(&self, index: BlockNumber) -> Option<H256> {
			self.numbers.get(&index).cloned()
		}

		fn block_receipts(&self, _hash: &H256) -> Option<BlockReceipts> {
			unimplemented!()
		}

		fn blocks_with_bloom(&self, _bloom: &Bloom, _from_block: BlockNumber, _to_block: BlockNumber) -> Vec<BlockNumber> {
			unimplemented!()
		}

		fn logs<F>(&self, _blocks: Vec<BlockNumber>, _matches: F, _limit: Option<usize>) -> Vec<LocalizedLogEntry>
			where F: Fn(&LogEntry) -> bool, Self: Sized {
			unimplemented!()
		}
	}

	fn basic_test(bytes: &[u8], engine: &EthEngine) -> Result<(), Error> {
		let header = BlockView::new(bytes).header();
		verify_block_basic(&header, bytes, engine)
	}

	fn family_test<BC>(bytes: &[u8], engine: &EthEngine, bc: &BC) -> Result<(), Error> where BC: BlockProvider {
		let view = BlockView::new(bytes);
		let header = view.header();
		let transactions: Vec<_> = view.transactions()
			.into_iter()
			.map(SignedTransaction::new)
			.collect::<Result<_,_>>()?;

		// TODO: client is really meant to be used for state query here by machine
		// additions that need access to state (tx filter in specific)
		// no existing tests need access to test, so having this not function
		// is fine.
		let client = ::client::TestBlockChainClient::default();

		let parent = bc.block_header(header.parent_hash())
			.ok_or(BlockError::UnknownParent(header.parent_hash().clone()))?;

		let full_params: FullFamilyParams = (
			bytes,
			&transactions[..],
			bc as &BlockProvider,
			&client as &::client::BlockChainClient
		);
		verify_block_family(&header, &parent, engine, Some(full_params))
	}

	fn unordered_test(bytes: &[u8], engine: &EthEngine) -> Result<(), Error> {
		let header = BlockView::new(bytes).header();
		verify_block_unordered(header, bytes.to_vec(), engine, false)?;
		Ok(())
	}

	#[test]
	fn test_verify_block() {
		use rlp::RlpStream;

		// Test against morden
		let mut good = Header::new();
		let spec = Spec::new_test();
		let engine = &*spec.engine;

		let min_gas_limit = engine.params().min_gas_limit;
		good.set_gas_limit(min_gas_limit);
		good.set_timestamp(40);
		good.set_number(10);

		let keypair = Random.generate().unwrap();

		let tr1 = Transaction {
			action: Action::Create,
			value: U256::from(0),
			data: Bytes::new(),
			gas: U256::from(30_000),
			gas_price: U256::from(40_000),
			nonce: U256::one()
		}.sign(keypair.secret(), None);

		let tr2 = Transaction {
			action: Action::Create,
			value: U256::from(0),
			data: Bytes::new(),
			gas: U256::from(30_000),
			gas_price: U256::from(40_000),
			nonce: U256::from(2)
		}.sign(keypair.secret(), None);

		let good_transactions = [ tr1.clone(), tr2.clone() ];

		let diff_inc = U256::from(0x40);

		let mut parent6 = good.clone();
		parent6.set_number(6);
		let mut parent7 = good.clone();
		parent7.set_number(7);
		parent7.set_parent_hash(parent6.hash());
		parent7.set_difficulty(parent6.difficulty().clone() + diff_inc);
		parent7.set_timestamp(parent6.timestamp() + 10);
		let mut parent8 = good.clone();
		parent8.set_number(8);
		parent8.set_parent_hash(parent7.hash());
		parent8.set_difficulty(parent7.difficulty().clone() + diff_inc);
		parent8.set_timestamp(parent7.timestamp() + 10);

		let mut good_uncle1 = good.clone();
		good_uncle1.set_number(9);
		good_uncle1.set_parent_hash(parent8.hash());
		good_uncle1.set_difficulty(parent8.difficulty().clone() + diff_inc);
		good_uncle1.set_timestamp(parent8.timestamp() + 10);
		good_uncle1.extra_data_mut().push(1u8);

		let mut good_uncle2 = good.clone();
		good_uncle2.set_number(8);
		good_uncle2.set_parent_hash(parent7.hash());
		good_uncle2.set_difficulty(parent7.difficulty().clone() + diff_inc);
		good_uncle2.set_timestamp(parent7.timestamp() + 10);
		good_uncle2.extra_data_mut().push(2u8);

		let good_uncles = vec![ good_uncle1.clone(), good_uncle2.clone() ];
		let mut uncles_rlp = RlpStream::new();
		uncles_rlp.append_list(&good_uncles);
		let good_uncles_hash = keccak(uncles_rlp.as_raw());
		let good_transactions_root = ordered_trie_root(good_transactions.iter().map(|t| ::rlp::encode::<UnverifiedTransaction>(t).into_vec()));

		let mut parent = good.clone();
		parent.set_number(9);
		parent.set_timestamp(parent8.timestamp() + 10);
		parent.set_parent_hash(parent8.hash());
		parent.set_difficulty(parent8.difficulty().clone() + diff_inc);

		good.set_parent_hash(parent.hash());
		good.set_difficulty(parent.difficulty().clone() + diff_inc);
		good.set_timestamp(parent.timestamp() + 10);

		let mut bc = TestBlockChain::new();
		bc.insert(create_test_block(&good));
		bc.insert(create_test_block(&parent));
		bc.insert(create_test_block(&parent6));
		bc.insert(create_test_block(&parent7));
		bc.insert(create_test_block(&parent8));

		check_ok(basic_test(&create_test_block(&good), engine));

		let mut header = good.clone();
		header.set_transactions_root(good_transactions_root.clone());
		header.set_uncles_hash(good_uncles_hash.clone());
		check_ok(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine));

		header.set_gas_limit(min_gas_limit - From::from(1));
		check_fail(basic_test(&create_test_block(&header), engine),
			InvalidGasLimit(OutOfBounds { min: Some(min_gas_limit), max: None, found: header.gas_limit().clone() }));

		header = good.clone();
		header.set_number(BlockNumber::max_value());
		check_fail(basic_test(&create_test_block(&header), engine),
			RidiculousNumber(OutOfBounds { max: Some(BlockNumber::max_value()), min: None, found: header.number() }));

		header = good.clone();
		let gas_used = header.gas_limit().clone() + 1.into();
		header.set_gas_used(gas_used);
		check_fail(basic_test(&create_test_block(&header), engine),
			TooMuchGasUsed(OutOfBounds { max: Some(header.gas_limit().clone()), min: None, found: header.gas_used().clone() }));

		header = good.clone();
		header.extra_data_mut().resize(engine.maximum_extra_data_size() + 1, 0u8);
		check_fail(basic_test(&create_test_block(&header), engine),
			ExtraDataOutOfBounds(OutOfBounds { max: Some(engine.maximum_extra_data_size()), min: None, found: header.extra_data().len() }));

		header = good.clone();
		header.extra_data_mut().resize(engine.maximum_extra_data_size() + 1, 0u8);
		check_fail(basic_test(&create_test_block(&header), engine),
			ExtraDataOutOfBounds(OutOfBounds { max: Some(engine.maximum_extra_data_size()), min: None, found: header.extra_data().len() }));

		header = good.clone();
		header.set_uncles_hash(good_uncles_hash.clone());
		check_fail(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine),
			InvalidTransactionsRoot(Mismatch { expected: good_transactions_root.clone(), found: header.transactions_root().clone() }));

		header = good.clone();
		header.set_transactions_root(good_transactions_root.clone());
		check_fail(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine),
			InvalidUnclesHash(Mismatch { expected: good_uncles_hash.clone(), found: header.uncles_hash().clone() }));

		check_ok(family_test(&create_test_block(&good), engine, &bc));
		check_ok(family_test(&create_test_block_with_data(&good, &good_transactions, &good_uncles), engine, &bc));

		header = good.clone();
		header.set_parent_hash(H256::random());
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine, &bc),
			UnknownParent(header.parent_hash().clone()));

		header = good.clone();
		header.set_timestamp(10);
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine, &bc),
			InvalidTimestamp(OutOfBounds { max: None, min: Some(parent.timestamp() + 1), found: header.timestamp() }));

		header = good.clone();
		header.set_timestamp(2450000000);
		check_fail_timestamp(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine), false);

		header = good.clone();
		header.set_timestamp(get_time().sec as u64 + 20);
		check_fail_timestamp(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine), true);

		header = good.clone();
		header.set_timestamp(get_time().sec as u64 + 10);
		header.set_uncles_hash(good_uncles_hash.clone());
		header.set_transactions_root(good_transactions_root.clone());
		check_ok(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine));

		header = good.clone();
		header.set_number(9);
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine, &bc),
			InvalidNumber(Mismatch { expected: parent.number() + 1, found: header.number() }));

		header = good.clone();
		let mut bad_uncles = good_uncles.clone();
		bad_uncles.push(good_uncle1.clone());
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &bad_uncles), engine, &bc),
			TooManyUncles(OutOfBounds { max: Some(engine.maximum_uncle_count(header.number())), min: None, found: bad_uncles.len() }));

		header = good.clone();
		bad_uncles = vec![ good_uncle1.clone(), good_uncle1.clone() ];
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &bad_uncles), engine, &bc),
			DuplicateUncle(good_uncle1.hash()));

		header = good.clone();
		header.set_gas_limit(0.into());
		header.set_difficulty("0000000000000000000000000000000000000000000000000000000000020000".parse::<U256>().unwrap());
		match family_test(&create_test_block(&header), engine, &bc) {
			Err(Error::Block(InvalidGasLimit(_))) => {},
			Err(_) => { panic!("should be invalid difficulty fail"); },
			_ => { panic!("Should be error, got Ok"); },
		}

		// TODO: some additional uncle checks
	}

	#[test]
	fn dust_protection() {
		use ethkey::{Generator, Random};
		use transaction::{Transaction, Action};
		use machine::EthereumMachine;
		use engines::NullEngine;

		let mut params = CommonParams::default();
		params.dust_protection_transition = 0;
		params.nonce_cap_increment = 2;

		let mut header = Header::default();
		header.set_number(1);

		let keypair = Random.generate().unwrap();
		let bad_transactions: Vec<_> = (0..3).map(|i| Transaction {
			action: Action::Create,
			value: U256::zero(),
			data: Vec::new(),
			gas: 0.into(),
			gas_price: U256::zero(),
			nonce: i.into(),
		}.sign(keypair.secret(), None)).collect();

		let good_transactions = [bad_transactions[0].clone(), bad_transactions[1].clone()];

		let machine = EthereumMachine::regular(params, BTreeMap::new());
		let engine = NullEngine::new(Default::default(), machine);
		check_fail(unordered_test(&create_test_block_with_data(&header, &bad_transactions, &[]), &engine), TooManyTransactions(keypair.address()));
		unordered_test(&create_test_block_with_data(&header, &good_transactions, &[]), &engine).unwrap();
	}
}
