/// Block and transaction verification functions
///
/// Block verification is done in 3 steps
/// 1. Quick verification upon adding to the block queue
/// 2. Signatures verification done in the queue.
/// 3. Final verification against the blockchain done before enactment.

use common::*;
use engine::Engine;
use blockchain::BlockChain;

/// Phase 1 quick block verification. Only does checks that are cheap. Operates on a single block
pub fn verify_block_basic(bytes: &[u8], engine: &Engine) -> Result<(), Error> {
	let block = BlockView::new(bytes);
	let header = block.header();
	try!(verify_header(&header));
	try!(verify_block_integrity(bytes, &header.transactions_root, &header.uncles_hash));
	try!(engine.verify_block_basic(&header, Some(bytes)));
	for u in Rlp::new(bytes).at(2).iter().map(|rlp| rlp.as_val::<Header>()) {
		try!(verify_header(&u));
		try!(engine.verify_block_basic(&u, None));
	}
	Ok(())
}

/// Phase 2 verification. Perform costly checks such as transaction signatures and block nonce for ethash.
/// Still operates on a individual block
/// TODO: return cached transactions, header hash.
pub fn verify_block_unordered(bytes: &[u8], engine: &Engine) -> Result<(), Error> {
	let block = BlockView::new(bytes);
	let header = block.header();
	try!(engine.verify_block_unordered(&header, Some(bytes)));
	for u in Rlp::new(bytes).at(2).iter().map(|rlp| rlp.as_val::<Header>()) {
		try!(engine.verify_block_unordered(&u, None));
	}
	Ok(())
}

/// Phase 3 verification. Check block information against parent and uncles.
pub fn verify_block_final(bytes: &[u8], engine: &Engine, bc: &BlockChain) -> Result<(), Error> {
	let block = BlockView::new(bytes);
	let header = block.header();
	let parent = try!(bc.block_header(&header.parent_hash).ok_or::<Error>(From::from(BlockError::UnknownParent(header.parent_hash.clone()))));
	try!(verify_parent(&header, &parent));
	try!(engine.verify_block_final(&header, &parent, Some(bytes)));

	let num_uncles = Rlp::new(bytes).at(2).item_count();
	if num_uncles != 0 {
		if num_uncles > engine.maximum_uncle_count() {
			return Err(From::from(BlockError::TooManyUncles(OutOfBounds { min: None, max: Some(engine.maximum_uncle_count()), found: num_uncles })));
		}

		let mut excluded = HashSet::new();
		excluded.insert(header.hash());
		let mut hash = header.parent_hash.clone();
		excluded.insert(hash.clone());
		for _ in 0..6 {
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
			let uncle_parent = try!(bc.block_header(&uncle.parent_hash).ok_or::<Error>(From::from(BlockError::UnknownUncleParent(uncle.parent_hash.clone()))));
			if excluded.contains(&uncle_parent.hash()) {
				return Err(From::from(BlockError::UncleInChain(uncle_parent.hash())))
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
			if depth > 6 {
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
			for _ in 0..depth {
				 expected_uncle_parent = bc.block_details(&expected_uncle_parent).unwrap().parent;
			}
			if expected_uncle_parent != uncle_parent.hash() {
				return Err(From::from(BlockError::UncleParentNotInChain(uncle_parent.hash())));
			}

			try!(engine.verify_block_final(&uncle, &uncle_parent, Some(bytes)));
		}
	}
	Ok(())
}

/// Check basic header parameters.
fn verify_header(header: &Header) -> Result<(), Error> {
	if header.number > From::from(BlockNumber::max_value()) {
		return Err(From::from(BlockError::InvalidNumber(OutOfBounds { max: Some(From::from(BlockNumber::max_value())), min: None, found: header.number })))
	}
	if header.gas_used > header.gas_limit {
		return Err(From::from(BlockError::TooMuchGasUsed(OutOfBounds { max: Some(header.gas_limit), min: None, found: header.gas_used })));
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
	if header.number <= parent.number {
		return Err(From::from(BlockError::InvalidNumber(OutOfBounds { max: None, min: Some(parent.number + 1), found: header.number })));
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

