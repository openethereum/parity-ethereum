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

//! Generalization of a state machine for a consensus engine.
//! This will define traits for the header, block, and state of a blockchain.

extern crate ethcore_util as util;

use util::{H256, U256};

/// A header. This contains important metadata about the block, as well as a
/// "seal" that indicates validity to a consensus engine.
pub trait Header {
	/// Cryptographic hash of the header, excluding the seal.
    fn bare_hash(&self) -> H256;

	/// Cryptographic hash of the header, including the seal.
    fn hash(&self) -> H256;

	/// Get a reference to the seal fields.
    fn seal(&self) -> &[Vec<u8>];
}

/// a header with an associated score (difficulty in PoW terms)
pub trait ScoredHeader {
	/// Get the score of this header.
    fn score(&self) -> U256;
}

/// the state machine the engine acquires consensus over.
/// Note that most of the definitions here actually relate to the _transition_ mechanism
/// as opposed to the state itself.
///
/// This is because consensus over transitions, as well as their ordering, is the most
/// important responsibility of the consensus engine.
pub trait Machine {
	/// The block header type.
    type Header: Header;
	/// The state type of the state machine.
    type State;
	/// Errors which can be returned during verification.
    type Error;

	// TODO verification functions.
	// verify transactions.
	// verify block basic
}
