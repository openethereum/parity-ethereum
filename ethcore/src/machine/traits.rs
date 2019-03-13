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

use ethereum_types::{U256, Address};
use block::ExecutedBlock;

/// Generalization of types surrounding blockchain-suitable state machines.
pub trait Machine: for<'a> LocalizedMachine<'a> {
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
	fn balance(&self, live: &ExecutedBlock, address: &Address) -> Result<U256, Self::Error>;

	/// Increment the balance of an account in the state of the live block.
	fn add_balance(&self, live: &mut ExecutedBlock, address: &Address, amount: &U256) -> Result<(), Self::Error>;
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
