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

use bytes::Bytes;
use ethereum_types::{H256, U256, Address};
use types::{
	transaction::{SignedTransaction, CallError},
	call_analytics::CallAnalytics,
	errors::EthcoreError as Error,
	errors::EthcoreResult,
	header::Header,
};

use block::{OpenBlock, SealedBlock, ClosedBlock};
use engine::Engine;
use machine::executed::Executed;
use account_state::state::StateInfo;

/// Provides `call` and `call_many` methods
pub trait Call {
	/// Type representing chain state
	type State: StateInfo;

	/// Makes a non-persistent transaction call.
	fn call(&self, tx: &SignedTransaction, analytics: CallAnalytics, state: &mut Self::State, header: &Header) -> Result<Executed, CallError>;

	/// Makes multiple non-persistent but dependent transaction calls.
	/// Returns a vector of successes or a failure if any of the transaction fails.
	fn call_many(&self, txs: &[(SignedTransaction, CallAnalytics)], state: &mut Self::State, header: &Header) -> Result<Vec<Executed>, CallError>;

	/// Estimates how much gas will be necessary for a call.
	fn estimate_gas(&self, t: &SignedTransaction, state: &Self::State, header: &Header) -> Result<U256, CallError>;
}

/// Provides `engine` method
pub trait EngineInfo {
	/// Get underlying engine object
	fn engine(&self) -> &dyn Engine;
}

/// Provides `reopen_block` method
pub trait ReopenBlock {
	/// Reopens an OpenBlock and updates uncles.
	fn reopen_block(&self, block: ClosedBlock) -> OpenBlock;
}

/// Provides `prepare_open_block` method
pub trait PrepareOpenBlock {
	/// Returns OpenBlock prepared for closing.
	fn prepare_open_block(&self,
		author: Address,
		gas_range_target: (U256, U256),
		extra_data: Bytes
	) -> Result<OpenBlock, Error>;
}

/// Provides methods used for sealing new state
pub trait BlockProducer: PrepareOpenBlock + ReopenBlock {}

///Provides `import_sealed_block` method
pub trait ImportSealedBlock {
	/// Import sealed block. Skips all verifications.
	fn import_sealed_block(&self, block: SealedBlock) -> EthcoreResult<H256>;
}

/// Provides `broadcast_proposal_block` method
pub trait BroadcastProposalBlock {
	/// Broadcast a block proposal.
	fn broadcast_proposal_block(&self, block: SealedBlock);
}

/// Provides methods to import sealed block and broadcast a block proposal
pub trait SealedBlockImporter: ImportSealedBlock + BroadcastProposalBlock {}
