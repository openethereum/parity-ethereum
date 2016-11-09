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

//! LES request types.

// TODO: make IPC compatible.

use util::H256;

/// A request for block headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Headers {
	/// Block information for the request being made.
	pub block: (u64, H256),
	/// The maximum amount of headers which can be returned.
	pub max: usize,
	/// The amount of headers to skip between each response entry.
	pub skip: usize,
	/// Whether the headers should proceed in falling number from the initial block.
	pub reverse: bool,
}

/// A request for specific block bodies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bodies {
	/// Hashes which bodies are being requested for.
	pub block_hashes: Vec<H256>
}

/// A request for transaction receipts.
///
/// This request is answered with a list of transaction receipts for each block
/// requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipts {
	/// Block hashes to return receipts for.
	pub block_hashes: Vec<H256>,
}

/// A request for a state proof
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateProof {
	/// Block hash to query state from.
	pub block: H256,
	/// Key of the state trie -- corresponds to account hash.
	pub key1: H256,
	/// Key in that account's storage trie; if empty, then the account RLP should be
	/// returned.
	pub key2: Option<H256>,
	/// if greater than zero, trie nodes beyond this level may be omitted.
	pub from_level: u32, // could even safely be u8; trie w/ 32-byte key can be at most 64-levels deep.
}

/// A request for state proofs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateProofs {
	/// All the proof requests.
	pub requests: Vec<StateProof>,
}

/// A request for contract code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractCodes {
	/// Block hash and account key (== sha3(address)) pairs to fetch code for.
	pub code_requests: Vec<(H256, H256)>,
}

/// A request for a header proof from the Canonical Hash Trie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderProof {
	/// Number of the CHT.
	pub cht_number: u64,
	/// Block number requested.
	pub block_number: u64,
	/// If greater than zero, trie nodes beyond this level may be omitted.
	pub from_level: u32,
}

/// A request for header proofs from the CHT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderProofs {
	/// All the proof requests.
	pub requests: Vec<HeaderProofs>,
}

/// Kinds of requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
	/// Requesting headers.
	Headers,
	/// Requesting block bodies.
	Bodies,
	/// Requesting transaction receipts.
	Receipts,
	/// Requesting proofs of state trie nodes.
	StateProofs,
	/// Requesting contract code by hash.
	Codes,
	/// Requesting header proofs (from the CHT).
	HeaderProofs,
}

/// Encompasses all possible types of requests in a single structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
	/// Requesting headers.
	Headers(Headers),
	/// Requesting block bodies.
	Bodies(Bodies),
	/// Requesting transaction receipts.
	Receipts(Receipts),
	/// Requesting state proofs.
	StateProofs(StateProofs),
	/// Requesting contract codes.
	Codes(ContractCodes),
	/// Requesting header proofs.
	HeaderProofs(HeaderProofs),
}

impl Request {
	/// Get the kind of request this is.
	pub fn kind(&self) -> Kind {
		match *self {
			Request::Headers(_) => Kind::Headers,
			Request::Bodies(_) => Kind::Bodies,
			Request::Receipts(_) => Kind::Receipts,
			Request::StateProofs(_) => Kind::StateProofs,
			Request::Codes(_) => Kind::Codes,
			Request::HeaderProofs(_) => Kind::HeaderProofs,
		}
	}
}