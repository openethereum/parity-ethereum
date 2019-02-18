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

//! Generalization of a state machine for a consensus engine.
//! This will define traits for the header, block, and state of a blockchain.

extern crate ethereum_types;

use ethereum_types::{H256, U256, Address};

/// A header. This contains important metadata about the block, as well as a
/// "seal" that indicates validity to a consensus engine.
pub trait Header {
	/// Cryptographic hash of the header, excluding the seal.
	fn bare_hash(&self) -> H256;

	/// Cryptographic hash of the header, including the seal.
	fn hash(&self) -> H256;

	/// Get a reference to the seal fields.
	fn seal(&self) -> &[Vec<u8>];

	/// The author of the header.
	fn author(&self) -> &Address;

	/// The number of the header.
	fn number(&self) -> u64;
}

/// A header with an associated score (difficulty in PoW terms)
pub trait ScoredHeader: Header {
	type Value;

	/// Get the score of this header.
	fn score(&self) -> &Self::Value;

	/// Set the score of this header.
	fn set_score(&mut self, score: Self::Value);
}

/// A header with associated total score.
pub trait TotalScoredHeader: Header {
	type Value;

	/// Get the total score of this header.
	fn total_score(&self) -> Self::Value;
}

/// A header with finalized information.
pub trait FinalizableHeader: Header {
	/// Get whether this header is considered finalized, so that it will never be replaced in reorganization.
	fn is_finalized(&self) -> bool;
}

/// A header with metadata information.
pub trait WithMetadataHeader: Header {
	/// Get the current header metadata.
	fn metadata(&self) -> Option<&[u8]>;
}

/// A "live" block is one which is in the process of the transition.
/// The state of this block can be mutated by arbitrary rules of the
/// state transition function.
pub trait LiveBlock: 'static {
	/// The block header type;
	type Header: Header;

	/// Get a reference to the header.
	fn header(&self) -> &Self::Header;

	/// Get a reference to the uncle headers. If the block type doesn't
	/// support uncles, return the empty slice.
	fn uncles(&self) -> &[Self::Header];
}

/// Trait for blocks which have a transaction type.
pub trait Transactions: LiveBlock {
	/// The transaction type.
	type Transaction;

	/// Get a reference to the transactions in this block.
	fn transactions(&self) -> &[Self::Transaction];
}

/// Generalization of types surrounding blockchain-suitable state machines.
pub trait Machine: for<'a> LocalizedMachine<'a> {
	/// The block header type.
	type Header: Header;
	/// The live block type.
	type LiveBlock: LiveBlock<Header=Self::Header>;
	/// Block header with metadata information.
	type ExtendedHeader: Header;
	/// A handle to a blockchain client for this machine.
	type EngineClient: ?Sized;
	/// A description of needed auxiliary data.
	type AuxiliaryRequest;
	/// Actions taken on ancestry blocks when commiting a new block.
	type AncestryAction;

	/// Errors which can occur when querying or interacting with the machine.
	type Error;

	/// Get the balance, in base units, associated with an account.
	/// Extracts data from the live block.
	fn balance(&self, live: &Self::LiveBlock, address: &Address) -> Result<U256, Self::Error>;

	/// Increment the balance of an account in the state of the live block.
	fn add_balance(&self, live: &mut Self::LiveBlock, address: &Address, amount: &U256) -> Result<(), Self::Error>;
}

/// Machine-related types localized to a specific lifetime.
// TODO: this is a workaround for a lack of associated type constructors in the language.
pub trait LocalizedMachine<'a>: Sync + Send {
	/// Definition of auxiliary data associated to a specific block.
	type AuxiliaryData: 'a;
	/// A context providing access to the state in a controlled capacity.
	/// Generally also provides verifiable proofs.
	type StateContext: ?Sized + 'a;
}
