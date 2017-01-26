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

//! LES request types.

use util::H256;

/// Either a hash or a number.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub enum HashOrNumber {
	/// Block hash variant.
	Hash(H256),
	/// Block number variant.
	Number(u64),
}

impl From<H256> for HashOrNumber {
	fn from(hash: H256) -> Self {
		HashOrNumber::Hash(hash)
	}
}

impl From<u64> for HashOrNumber {
	fn from(num: u64) -> Self {
		HashOrNumber::Number(num)
	}
}

/// A request for block headers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct Headers {
	/// Starting block number or hash.
	pub start: HashOrNumber,
	/// The maximum amount of headers which can be returned.
	pub max: usize,
	/// The amount of headers to skip between each response entry.
	pub skip: u64,
	/// Whether the headers should proceed in falling number from the initial block.
	pub reverse: bool,
}

/// A request for specific block bodies.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct Bodies {
	/// Hashes which bodies are being requested for.
	pub block_hashes: Vec<H256>
}

/// A request for transaction receipts.
///
/// This request is answered with a list of transaction receipts for each block
/// requested.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct Receipts {
	/// Block hashes to return receipts for.
	pub block_hashes: Vec<H256>,
}

/// A request for a state proof
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
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
#[cfg_attr(feature = "ipc", binary)]
pub struct StateProofs {
	/// All the proof requests.
	pub requests: Vec<StateProof>,
}

/// A request for contract code.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct ContractCode {
	/// Block hash
	pub block_hash: H256,
	/// Account key (== sha3(address))
	pub account_key: H256,
}

/// A request for contract code.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct ContractCodes {
	/// Block hash and account key (== sha3(address)) pairs to fetch code for.
	pub code_requests: Vec<ContractCode>,
}

/// A request for a header proof from the Canonical Hash Trie.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct HeaderProof {
	/// Number of the CHT.
	pub cht_number: u64,
	/// Block number requested. May not be 0: genesis isn't included in any CHT.
	pub block_number: u64,
	/// If greater than zero, trie nodes beyond this level may be omitted.
	pub from_level: u32,
}

/// A request for header proofs from the CHT.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub struct HeaderProofs {
	/// All the proof requests.
	pub requests: Vec<HeaderProof>,
}

/// Kinds of requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
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
#[cfg_attr(feature = "ipc", binary)]
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

	/// Get the amount of requests being made.
	pub fn amount(&self) -> usize {
		match *self {
			Request::Headers(ref req) => req.max,
			Request::Bodies(ref req) => req.block_hashes.len(),
			Request::Receipts(ref req) => req.block_hashes.len(),
			Request::StateProofs(ref req) => req.requests.len(),
			Request::Codes(ref req) => req.code_requests.len(),
			Request::HeaderProofs(ref req) => req.requests.len(),
		}
	}
}
