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

//! Merkle proof request.
use util::hash::H256;

/// A request for a state merkle proof.
#[derive(Debug, Clone, PartialEq, Eq, Binary)]
pub enum ProofRequest {
	/// Request a proof of the given account's (denoted by sha3(address))
	/// node in the state trie. Nodes with depth less than the second item
	/// may be omitted.
	Account(H256, usize),

	/// Request a proof for a key in the given account's storage trie.
	/// Both values are hashes of their actual values. Nodes with depth
	/// less than the third item may be omitted.
	Storage(H256, H256, usize),
}

/// A request for a Canonical Hash Trie proof for the given block number.
/// Nodes with depth less than the second item may be omitted.
#[derive(Debug, Clone, PartialEq, Eq, Binary)]
pub struct CHTProofRequest {
	/// The number of the block the proof is requested for.
	/// The CHT's number can be deduced from this (`number` / 4096)
	pub number: u64,

	/// Nodes with depth less than this can be omitted from the proof.
	pub depth: usize,
}