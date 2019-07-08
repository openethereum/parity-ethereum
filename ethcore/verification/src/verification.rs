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

use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use keccak_hash::keccak;
use rlp::Rlp;
use triehash::ordered_trie_root;
use unexpected::{Mismatch, OutOfBounds};

use ethcore_blockchain::BlockProvider;
use call_contract::CallContract;
use client_traits::BlockInfo;

//use engines::{Engine, MAX_UNCLE_AGE};
use crate::error::Error;
use common_types::{
	BlockNumber,
	header::Header,
	block::{BlockError, PreverifiedBlock},
};
//use verification::queue::kind::blocks::Unverified;

use time_utils::CheckedSystemTime;

/// Phase 1 quick block verification. Only does checks that are cheap. Operates on a single block
pub fn verify_block_basic(block: &Unverified, engine: &dyn Engine, check_seal: bool) -> Result<(), Error> {
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
pub fn verify_block_unordered(block: Unverified, engine: &dyn Engine, check_seal: bool) -> Result<PreverifiedBlock, Error> {
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
		.collect::<Result<Vec<_>, Error>>()?;

	Ok(PreverifiedBlock {
		header,
		transactions,
		uncles: block.uncles,
		bytes: block.bytes,
	})
}

/// Parameters for full verification of block family
pub struct FullFamilyParams<'a, C: BlockInfo + CallContract + 'a> {
	/// Pre-verified block
	pub block: &'a PreverifiedBlock,

	/// Block provider to use during verification
	pub block_provider: &'a dyn BlockProvider,

	/// Engine client to use during verification
	pub client: &'a C,
}
