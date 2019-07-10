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

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use keccak_hash::keccak;
use rlp::Rlp;
use triehash_ethereum::ordered_trie_root;
use unexpected::{Mismatch, OutOfBounds};

use ethcore_blockchain::BlockProvider;
use call_contract::CallContract;
use client_traits::{BlockInfo, VerifyingEngine};

use crate::{
	queue::kind::blocks::Unverified,
};
use common_types::{
	BlockNumber,
	header::Header,
	block::PreverifiedBlock,
	errors::{BlockError, EthcoreError},
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
pub fn verify_block_unordered(block: Unverified, engine: &dyn VerifyingEngine, check_seal: bool) -> Result<PreverifiedBlock, EthcoreError> {
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
