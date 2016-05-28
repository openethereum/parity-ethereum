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

/// Block and transaction verification functions
///
/// Block verification is done in 3 steps
/// 1. Quick verification upon adding to the block queue
/// 2. Signatures verification done in the queue.
/// 3. Final verification against the blockchain done before enactment.

use common::*;
use engine::Engine;
use blockchain::*;

/// Preprocessed block data gathered in `verify_block_unordered` call
pub struct PreverifiedBlock {
	/// Populated block header
	pub header: Header,
	/// Populated block transactions
	pub transactions: Vec<SignedTransaction>,
	/// Block bytes
	pub bytes: Bytes,
}

/// Phase 1 quick block verification. Only does checks that are cheap. Operates on a single block
pub fn verify_block_basic(header: &Header, bytes: &[u8], engine: &Engine) -> Result<(), Error> {
	try!(verify_header(&header, engine));
	try!(verify_block_integrity(bytes, &header.transactions_root, &header.uncles_hash));
	try!(engine.verify_block_basic(&header, Some(bytes)));
	for u in Rlp::new(bytes).at(2).iter().map(|rlp| rlp.as_val::<Header>()) {
		try!(verify_header(&u, engine));
		try!(engine.verify_block_basic(&u, None));
	}
	// Verify transactions.
	// TODO: either use transaction views or cache the decoded transactions.
	let v = BlockView::new(bytes);
	for t in v.transactions() {
		try!(engine.verify_transaction_basic(&t, &header));
	}
	Ok(())
}

/// Phase 2 verification. Perform costly checks such as transaction signatures and block nonce for ethash.
/// Still operates on a individual block
/// Returns a `PreverifiedBlock` structure populated with transactions
pub fn verify_block_unordered(header: Header, bytes: Bytes, engine: &Engine) -> Result<PreverifiedBlock, Error> {
	try!(engine.verify_block_unordered(&header, Some(&bytes)));
	for u in Rlp::new(&bytes).at(2).iter().map(|rlp| rlp.as_val::<Header>()) {
		try!(engine.verify_block_unordered(&u, None));
	}
	// Verify transactions.
	let mut transactions = Vec::new();
	{
		let v = BlockView::new(&bytes);
		for t in v.transactions() {
			try!(engine.verify_transaction(&t, &header));
			transactions.push(t);
		}
	}
	Ok(PreverifiedBlock {
		header: header,
		transactions: transactions,
		bytes: bytes,
	})
}

/// Phase 3 verification. Check block information against parent and uncles.
pub fn verify_block_family(header: &Header, bytes: &[u8], engine: &Engine, bc: &BlockProvider) -> Result<(), Error> {
	// TODO: verify timestamp
	let parent = try!(bc.block_header(&header.parent_hash).ok_or_else(|| Error::from(BlockError::UnknownParent(header.parent_hash.clone()))));
	try!(verify_parent(&header, &parent));
	try!(engine.verify_block_family(&header, &parent, Some(bytes)));

	let num_uncles = Rlp::new(bytes).at(2).item_count();
	if num_uncles != 0 {
		if num_uncles > engine.maximum_uncle_count() {
			return Err(From::from(BlockError::TooManyUncles(OutOfBounds { min: None, max: Some(engine.maximum_uncle_count()), found: num_uncles })));
		}

		let mut excluded = HashSet::new();
		excluded.insert(header.hash());
		let mut hash = header.parent_hash.clone();
		excluded.insert(hash.clone());
		for _ in 0..engine.maximum_uncle_age() {
			match bc.block_details(&hash) {
				Some(details) => {
					excluded.insert(details.parent.clone());
					let b = bc.block(&hash).unwrap();
					excluded.extend(BlockView::new(&b).uncle_hashes());
					hash = details.parent;
				}
				None => break
			}
		}

		for uncle in Rlp::new(bytes).at(2).iter().map(|rlp| rlp.as_val::<Header>()) {
			if excluded.contains(&uncle.hash()) {
				return Err(From::from(BlockError::UncleInChain(uncle.hash())))
			}

			// m_currentBlock.number() - uncle.number()		m_cB.n - uP.n()
			// 1											2
			// 2
			// 3
			// 4
			// 5
			// 6											7
			//												(8 Invalid)

			let depth = if header.number > uncle.number { header.number - uncle.number } else { 0 };
			if depth > engine.maximum_uncle_age() as u64 {
				return Err(From::from(BlockError::UncleTooOld(OutOfBounds { min: Some(header.number - depth), max: Some(header.number - 1), found: uncle.number })));
			}
			else if depth < 1 {
				return Err(From::from(BlockError::UncleIsBrother(OutOfBounds { min: Some(header.number - depth), max: Some(header.number - 1), found: uncle.number })));
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
			let mut expected_uncle_parent = header.parent_hash.clone();
			let uncle_parent = try!(bc.block_header(&uncle.parent_hash).ok_or_else(|| Error::from(BlockError::UnknownUncleParent(uncle.parent_hash.clone()))));
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

			try!(verify_parent(&uncle, &uncle_parent));
			try!(engine.verify_block_family(&uncle, &uncle_parent, Some(bytes)));
		}
	}
	Ok(())
}

/// Phase 4 verification. Check block information against transaction enactment results,
pub fn verify_block_final(expected: &Header, got: &Header) -> Result<(), Error> {
	if expected.gas_used != got.gas_used {
		return Err(From::from(BlockError::InvalidGasUsed(Mismatch { expected: expected.gas_used, found: got.gas_used })))
	}
	if expected.log_bloom != got.log_bloom {
		return Err(From::from(BlockError::InvalidLogBloom(Mismatch { expected: expected.log_bloom.clone(), found: got.log_bloom.clone() })))
	}
	if expected.state_root != got.state_root {
		return Err(From::from(BlockError::InvalidStateRoot(Mismatch { expected: expected.state_root.clone(), found: got.state_root.clone() })))
	}
	if expected.receipts_root != got.receipts_root {
		return Err(From::from(BlockError::InvalidReceiptsRoot(Mismatch { expected: expected.receipts_root.clone(), found: got.receipts_root.clone() })))
	}
	Ok(())
}

/// Check basic header parameters.
fn verify_header(header: &Header, engine: &Engine) -> Result<(), Error> {
	if header.number >= From::from(BlockNumber::max_value()) {
		return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { max: Some(From::from(BlockNumber::max_value())), min: None, found: header.number })))
	}
	if header.gas_used > header.gas_limit {
		return Err(From::from(BlockError::TooMuchGasUsed(OutOfBounds { max: Some(header.gas_limit), min: None, found: header.gas_used })));
	}
	let min_gas_limit = engine.params().min_gas_limit;
	if header.gas_limit < min_gas_limit {
		return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas_limit), max: None, found: header.gas_limit })));
	}
	let maximum_extra_data_size = engine.maximum_extra_data_size();
	if header.number != 0 && header.extra_data.len() > maximum_extra_data_size {
		return Err(From::from(BlockError::ExtraDataOutOfBounds(OutOfBounds { min: None, max: Some(maximum_extra_data_size), found: header.extra_data.len() })));
	}
	Ok(())
}

/// Check header parameters agains parent header.
fn verify_parent(header: &Header, parent: &Header) -> Result<(), Error> {
	if !header.parent_hash.is_zero() && parent.hash() != header.parent_hash {
		return Err(From::from(BlockError::InvalidParentHash(Mismatch { expected: parent.hash(), found: header.parent_hash.clone() })))
	}
	if header.timestamp <= parent.timestamp {
		return Err(From::from(BlockError::InvalidTimestamp(OutOfBounds { max: None, min: Some(parent.timestamp + 1), found: header.timestamp })))
	}
	if header.number != parent.number + 1 {
		return Err(From::from(BlockError::InvalidNumber(Mismatch { expected: parent.number + 1, found: header.number })));
	}
	Ok(())
}

/// Verify block data against header: transactions root and uncles hash.
fn verify_block_integrity(block: &[u8], transactions_root: &H256, uncles_hash: &H256) -> Result<(), Error> {
	let block = Rlp::new(block);
	let tx = block.at(1);
	let expected_root = &ordered_trie_root(tx.iter().map(|r| r.as_raw().to_vec()).collect()); //TODO: get rid of vectors here
	if expected_root != transactions_root {
		return Err(From::from(BlockError::InvalidTransactionsRoot(Mismatch { expected: expected_root.clone(), found: transactions_root.clone() })))
	}
	let expected_uncles = &block.at(2).as_raw().sha3();
	if expected_uncles != uncles_hash {
		return Err(From::from(BlockError::InvalidUnclesHash(Mismatch { expected: expected_uncles.clone(), found: uncles_hash.clone() })))
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use util::*;
	use header::*;
	use verification::*;
	use blockchain::extras::*;
	use error::*;
	use error::BlockError::*;
	use views::*;
	use blockchain::*;
	use engine::*;
	use spec::*;
	use transaction::*;
	use tests::helpers::*;

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
			let hash = BlockView::new(&bytes).header_view().sha3();
			self.blocks.insert(hash.clone(), bytes);
			self.numbers.insert(number, hash.clone());
		}
	}

	impl BlockProvider for TestBlockChain {
		fn is_known(&self, hash: &H256) -> bool {
			self.blocks.contains_key(hash)
		}

		/// Get raw block data
		fn block(&self, hash: &H256) -> Option<Bytes> {
			self.blocks.get(hash).cloned()
		}

		/// Get the familial details concerning a block.
		fn block_details(&self, hash: &H256) -> Option<BlockDetails> {
			self.blocks.get(hash).map(|bytes| {
				let header = BlockView::new(bytes).header();
				BlockDetails {
					number: header.number,
					total_difficulty: header.difficulty,
					parent: header.parent_hash,
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

		fn blocks_with_bloom(&self, _bloom: &H2048, _from_block: BlockNumber, _to_block: BlockNumber) -> Vec<BlockNumber> {
			unimplemented!()
		}

		fn block_receipts(&self, _hash: &H256) -> Option<BlockReceipts> {
			unimplemented!()
		}
	}

	fn basic_test(bytes: &[u8], engine: &Engine) -> Result<(), Error> {
		let header = BlockView::new(bytes).header();
		verify_block_basic(&header, bytes, engine)
	}

	fn family_test<BC>(bytes: &[u8], engine: &Engine, bc: &BC) -> Result<(), Error> where BC: BlockProvider {
		let header = BlockView::new(bytes).header();
		verify_block_family(&header, bytes, engine, bc)
	}

	#[test]
	#[cfg_attr(feature="dev", allow(similar_names))]
	fn test_verify_block() {
		// Test against morden
		let mut good = Header::new();
		let spec = Spec::new_test();
		let engine = &spec.engine;

		let min_gas_limit = engine.params().min_gas_limit;
		good.gas_limit = min_gas_limit;
		good.timestamp = 40;
		good.number = 10;

		let keypair = KeyPair::create().unwrap();

		let tr1 = Transaction {
			action: Action::Create,
			value: U256::from(0),
			data: Bytes::new(),
			gas: U256::from(30_000),
			gas_price: U256::from(40_000),
			nonce: U256::one()
		}.sign(&keypair.secret());

		let tr2 = Transaction {
			action: Action::Create,
			value: U256::from(0),
			data: Bytes::new(),
			gas: U256::from(30_000),
			gas_price: U256::from(40_000),
			nonce: U256::from(2)
		}.sign(&keypair.secret());

		let good_transactions = [ &tr1, &tr2 ];

		let diff_inc = U256::from(0x40);

		let mut parent6 = good.clone();
		parent6.number = 6;
		let mut parent7 = good.clone();
		parent7.number = 7;
		parent7.parent_hash = parent6.hash();
		parent7.difficulty = parent6.difficulty + diff_inc;
		parent7.timestamp = parent6.timestamp + 10;
		let mut parent8 = good.clone();
		parent8.number = 8;
		parent8.parent_hash = parent7.hash();
		parent8.difficulty = parent7.difficulty + diff_inc;
		parent8.timestamp = parent7.timestamp + 10;

		let mut good_uncle1 = good.clone();
		good_uncle1.number = 9;
		good_uncle1.parent_hash = parent8.hash();
		good_uncle1.difficulty = parent8.difficulty + diff_inc;
		good_uncle1.timestamp = parent8.timestamp + 10;
		good_uncle1.extra_data.push(1u8);

		let mut good_uncle2 = good.clone();
		good_uncle2.number = 8;
		good_uncle2.parent_hash = parent7.hash();
		good_uncle2.difficulty = parent7.difficulty + diff_inc;
		good_uncle2.timestamp = parent7.timestamp + 10;
		good_uncle2.extra_data.push(2u8);

		let good_uncles = vec![ good_uncle1.clone(), good_uncle2.clone() ];
		let mut uncles_rlp = RlpStream::new();
		uncles_rlp.append(&good_uncles);
		let good_uncles_hash = uncles_rlp.as_raw().sha3();
		let good_transactions_root = ordered_trie_root(good_transactions.iter().map(|t| encode::<SignedTransaction>(t).to_vec()).collect());

		let mut parent = good.clone();
		parent.number = 9;
		parent.timestamp = parent8.timestamp + 10;
		parent.parent_hash = parent8.hash();
		parent.difficulty = parent8.difficulty + diff_inc;

		good.parent_hash = parent.hash();
		good.difficulty = parent.difficulty + diff_inc;
		good.timestamp = parent.timestamp + 10;

		let mut bc = TestBlockChain::new();
		bc.insert(create_test_block(&good));
		bc.insert(create_test_block(&parent));
		bc.insert(create_test_block(&parent6));
		bc.insert(create_test_block(&parent7));
		bc.insert(create_test_block(&parent8));

		check_ok(basic_test(&create_test_block(&good), engine.deref()));

		let mut header = good.clone();
		header.transactions_root = good_transactions_root.clone();
		header.uncles_hash = good_uncles_hash.clone();
		check_ok(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine.deref()));

		header.gas_limit = min_gas_limit - From::from(1);
		check_fail(basic_test(&create_test_block(&header), engine.deref()),
			InvalidGasLimit(OutOfBounds { min: Some(min_gas_limit), max: None, found: header.gas_limit }));

		header = good.clone();
		header.number = BlockNumber::max_value();
		check_fail(basic_test(&create_test_block(&header), engine.deref()),
			RidiculousNumber(OutOfBounds { max: Some(BlockNumber::max_value()), min: None, found: header.number }));

		header = good.clone();
		header.gas_used = header.gas_limit + From::from(1);
		check_fail(basic_test(&create_test_block(&header), engine.deref()),
			TooMuchGasUsed(OutOfBounds { max: Some(header.gas_limit), min: None, found: header.gas_used }));

		header = good.clone();
		header.extra_data.resize(engine.maximum_extra_data_size() + 1, 0u8);
		check_fail(basic_test(&create_test_block(&header), engine.deref()),
			ExtraDataOutOfBounds(OutOfBounds { max: Some(engine.maximum_extra_data_size()), min: None, found: header.extra_data.len() }));

		header = good.clone();
		header.extra_data.resize(engine.maximum_extra_data_size() + 1, 0u8);
		check_fail(basic_test(&create_test_block(&header), engine.deref()),
			ExtraDataOutOfBounds(OutOfBounds { max: Some(engine.maximum_extra_data_size()), min: None, found: header.extra_data.len() }));

		header = good.clone();
		header.uncles_hash = good_uncles_hash.clone();
		check_fail(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine.deref()),
			InvalidTransactionsRoot(Mismatch { expected: good_transactions_root.clone(), found: header.transactions_root }));

		header = good.clone();
		header.transactions_root = good_transactions_root.clone();
		check_fail(basic_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine.deref()),
			InvalidUnclesHash(Mismatch { expected: good_uncles_hash.clone(), found: header.uncles_hash }));

		check_ok(family_test(&create_test_block(&good), engine.deref(), &bc));
		check_ok(family_test(&create_test_block_with_data(&good, &good_transactions, &good_uncles), engine.deref(), &bc));

		header = good.clone();
		header.parent_hash = H256::random();
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine.deref(), &bc),
			UnknownParent(header.parent_hash));

		header = good.clone();
		header.timestamp = 10;
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine.deref(), &bc),
			InvalidTimestamp(OutOfBounds { max: None, min: Some(parent.timestamp + 1), found: header.timestamp }));

		header = good.clone();
		header.number = 9;
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &good_uncles), engine.deref(), &bc),
			InvalidNumber(Mismatch { expected: parent.number + 1, found: header.number }));

		header = good.clone();
		let mut bad_uncles = good_uncles.clone();
		bad_uncles.push(good_uncle1.clone());
		check_fail(family_test(&create_test_block_with_data(&header, &good_transactions, &bad_uncles), engine.deref(), &bc),
			TooManyUncles(OutOfBounds { max: Some(engine.maximum_uncle_count()), min: None, found: bad_uncles.len() }));

		// TODO: some additional uncle checks
	}
}
