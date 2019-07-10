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

//! Block and transaction verification functions
//!
//! Block verification is done in 3 steps
//! 1. Quick verification upon adding to the block queue
//! 2. Signatures verification done in the queue.
//! 3. Final verification against the blockchain done before enactment.

use std::{
	collections::HashSet,
	time::{Duration, SystemTime, UNIX_EPOCH},
};

use keccak_hash::keccak;
use rlp::Rlp;
use triehash_ethereum::ordered_trie_root;
use unexpected::{Mismatch, OutOfBounds};

use ethcore_blockchain::BlockProvider;
use call_contract::CallContract;
use client_traits::{BlockInfo, VerifyingEngine, VerifyingClient};

use crate::{
	queue::kind::blocks::Unverified,
};
use common_types::{
	BlockNumber,
	header::Header,
	block::PreverifiedBlock,
	errors::{BlockError, EthcoreError},
	engines::MAX_UNCLE_AGE,
};

use time_utils::CheckedSystemTime;

/// Parameters for full verification of block family
pub struct FullFamilyParams<'a, C: BlockInfo + CallContract + 'a> {
	/// Pre-verified block
	pub block: &'a PreverifiedBlock,

	/// Block provider to use during verification
	pub block_provider: &'a dyn BlockProvider,

	/// Engine client to use during verification
	pub client: &'a C,
}

/// Phase 1 quick block verification. Only does checks that are cheap. Operates on a single block
pub fn verify_block_basic(block: &Unverified, engine: &dyn VerifyingEngine, check_seal: bool) -> Result<(), EthcoreError> {
	verify_header_params(&block.header, engine, true, check_seal)?;
	verify_block_integrity(block)?;

	if check_seal {
		engine.verify_block_basic(&block.header)?;
	}

	for uncle in &block.uncles {
		verify_header_params(uncle, engine, false, check_seal)?;
		if check_seal {
			engine.verify_block_basic(uncle)?;
		}
	}

	for t in &block.transactions {
		engine.verify_transaction_basic(t, &block.header)?;
	}

	Ok(())
}

/// Phase 2 verification. Perform costly checks such as transaction signatures and block nonce for ethash.
/// Still operates on a individual block
/// Returns a `PreverifiedBlock` structure populated with transactions
pub fn verify_block_unordered(
	block: Unverified,
	engine: &dyn VerifyingEngine,
	check_seal: bool
) -> Result<PreverifiedBlock, EthcoreError> {
	let header = block.header;
	if check_seal {
		engine.verify_block_unordered(&header)?;
		for uncle in &block.uncles {
			engine.verify_block_unordered(uncle)?;
		}
	}
	// Verify transactions.
	let nonce_cap = if header.number() >= engine.params().dust_protection_transition {
		Some((engine.params().nonce_cap_increment * header.number()).into())
	} else {
		None
	};

	let transactions = block.transactions
		.into_iter()
		.map(|t| {
			let t = engine.verify_transaction_unordered(t, &header)?;
			if let Some(max_nonce) = nonce_cap {
				if t.nonce >= max_nonce {
					return Err(BlockError::TooManyTransactions(t.sender()).into());
				}
			}
			Ok(t)
		})
		.collect::<Result<Vec<_>, EthcoreError>>()?;

	Ok(PreverifiedBlock {
		header,
		transactions,
		uncles: block.uncles,
		bytes: block.bytes,
	})
}

/// Phase 3 verification. Check block information against parent and uncles.
//pub fn verify_block_family<C: BlockInfo + CallContract>(
pub fn verify_block_family<C: VerifyingClient>(
	header: &Header,
	parent: &Header,
	engine: &dyn VerifyingEngine,
	do_full: Option<FullFamilyParams<C>>
) -> Result<(), EthcoreError> {
	// TODO: verify timestamp
	verify_parent(&header, &parent, engine)?;
	engine.verify_block_family(&header, &parent)?;

	let params = match do_full {
		Some(x) => x,
		None => return Ok(()),
	};

	verify_uncles(params.block, params.block_provider, engine)?;

	// transactions are verified against the parent header since the current
	// state wasn't available when the tx was created
	engine.verify_transactions(&params.block.transactions, parent, params.client)?;
//	for tx in &params.block.transactions {
//		// transactions are verified against the parent header since the current
//		// state wasn't available when the tx was created
//		engine.machine().verify_transaction(tx, parent, params.client)?;
//	}

	Ok(())
}


/// Phase 4 verification. Check block information against transaction enactment results,
pub fn verify_block_final(
	expected: &Header,
	got: &Header
) -> Result<(), EthcoreError> {
	if expected.state_root() != got.state_root() {
		return Err(From::from(BlockError::InvalidStateRoot(Mismatch { expected: *expected.state_root(), found: *got.state_root() })))
	}
	if expected.gas_used() != got.gas_used() {
		return Err(From::from(BlockError::InvalidGasUsed(Mismatch { expected: *expected.gas_used(), found: *got.gas_used() })))
	}
	if expected.log_bloom() != got.log_bloom() {
		return Err(From::from(BlockError::InvalidLogBloom(Box::new(Mismatch { expected: *expected.log_bloom(), found: *got.log_bloom() }))))
	}
	if expected.receipts_root() != got.receipts_root() {
		return Err(From::from(BlockError::InvalidReceiptsRoot(Mismatch { expected: *expected.receipts_root(), found: *got.receipts_root() })))
	}
	Ok(())
}

/// Check basic header parameters.
pub fn verify_header_params(header: &Header, engine: &dyn VerifyingEngine, is_full: bool, check_seal: bool) -> Result<(), EthcoreError> {
	if check_seal {
		let expected_seal_fields = engine.seal_fields(header);
		if header.seal().len() != expected_seal_fields {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: expected_seal_fields, found: header.seal().len() }
			)));
		}
	}

	if header.number() >= From::from(BlockNumber::max_value()) {
		return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { max: Some(From::from(BlockNumber::max_value())), min: None, found: header.number() })))
	}
	if header.gas_used() > header.gas_limit() {
		return Err(From::from(BlockError::TooMuchGasUsed(OutOfBounds { max: Some(*header.gas_limit()), min: None, found: *header.gas_used() })));
	}
	let min_gas_limit = engine.params().min_gas_limit;
	if header.gas_limit() < &min_gas_limit {
		return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas_limit), max: None, found: *header.gas_limit() })));
	}
	if let Some(limit) = engine.maximum_gas_limit() {
		if header.gas_limit() > &limit {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: None, max: Some(limit), found: *header.gas_limit() })));
		}
	}
	let maximum_extra_data_size = engine.maximum_extra_data_size();
	if header.number() != 0 && header.extra_data().len() > maximum_extra_data_size {
		return Err(From::from(BlockError::ExtraDataOutOfBounds(OutOfBounds { min: None, max: Some(maximum_extra_data_size), found: header.extra_data().len() })));
	}

	if let Some(ext) = engine.ethash_extensions() {
		if header.number() >= ext.dao_hardfork_transition &&
			header.number() <= ext.dao_hardfork_transition + 9 &&
			header.extra_data()[..] != b"dao-hard-fork"[..] {
			return Err(From::from(BlockError::ExtraDataOutOfBounds(OutOfBounds { min: None, max: None, found: 0 })));
		}
	}

	if is_full {
		const ACCEPTABLE_DRIFT: Duration = Duration::from_secs(15);
		// this will resist overflow until `year 2037`
		let max_time = SystemTime::now() + ACCEPTABLE_DRIFT;
		let invalid_threshold = max_time + ACCEPTABLE_DRIFT * 9;
		let timestamp = CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(header.timestamp()))
			.ok_or(BlockError::TimestampOverflow)?;

		if timestamp > invalid_threshold {
			return Err(From::from(BlockError::InvalidTimestamp(OutOfBounds { max: Some(max_time), min: None, found: timestamp }.into())))
		}

		if timestamp > max_time {
			return Err(From::from(BlockError::TemporarilyInvalid(OutOfBounds { max: Some(max_time), min: None, found: timestamp }.into())))
		}
	}

	Ok(())
}

/// Check header parameters agains parent header.
fn verify_parent(header: &Header, parent: &Header, engine: &dyn VerifyingEngine) -> Result<(), EthcoreError> {
	assert!(header.parent_hash().is_zero() || &parent.hash() == header.parent_hash(),
	        "Parent hash should already have been verified; qed");

	let gas_limit_divisor = engine.params().gas_limit_bound_divisor;

	if !engine.is_timestamp_valid(header.timestamp(), parent.timestamp()) {
		let now = SystemTime::now();
		let min = CheckedSystemTime::checked_add(now, Duration::from_secs(parent.timestamp().saturating_add(1)))
			.ok_or(BlockError::TimestampOverflow)?;
		let found = CheckedSystemTime::checked_add(now, Duration::from_secs(header.timestamp()))
			.ok_or(BlockError::TimestampOverflow)?;
		return Err(From::from(BlockError::InvalidTimestamp(OutOfBounds { max: None, min: Some(min), found }.into())))
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
		return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: *header.gas_limit() })));
	}

	Ok(())
}


fn verify_uncles(
	block: &PreverifiedBlock,
	bc: &dyn BlockProvider,
	engine: &dyn VerifyingEngine
) -> Result<(), EthcoreError> {
	let header = &block.header;
	let num_uncles = block.uncles.len();
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
		for _ in 0..MAX_UNCLE_AGE {
			match bc.block_details(&hash) {
				Some(details) => {
					excluded.insert(details.parent);
					let b = bc.block(&hash)
						.expect("parent already known to be stored; qed");
					excluded.extend(b.uncle_hashes());
					hash = details.parent;
				}
				None => break
			}
		}

		let mut verified = HashSet::new();
		for uncle in &block.uncles {
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
			if depth > MAX_UNCLE_AGE as u64 {
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
			let uncle_parent = bc.block_header_data(&uncle.parent_hash())
				.ok_or_else(|| EthcoreError::from(BlockError::UnknownUncleParent(uncle.parent_hash().clone())))?;
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

			let uncle_parent = uncle_parent.decode()?;
			verify_parent(&uncle, &uncle_parent, engine)?;
			engine.verify_block_family(&uncle, &uncle_parent)?;
			verified.insert(uncle.hash());
		}
	}

	Ok(())
}

/// Verify block data against header: transactions root and uncles hash.
fn verify_block_integrity(block: &Unverified) -> Result<(), EthcoreError> {
	let block_rlp = Rlp::new(&block.bytes);
	let tx = block_rlp.at(1)?;
	let expected_root = ordered_trie_root(tx.iter().map(|r| r.as_raw()));
	if &expected_root != block.header.transactions_root() {
		return Err(BlockError::InvalidTransactionsRoot(Mismatch {
			expected: expected_root,
			found: *block.header.transactions_root(),
		}).into());
	}
	let expected_uncles = keccak(block_rlp.at(2)?.as_raw());
	if &expected_uncles != block.header.uncles_hash(){
		return Err(BlockError::InvalidUnclesHash(Mismatch {
			expected: expected_uncles,
			found: *block.header.uncles_hash(),
		}).into());
	}
	Ok(())
}


