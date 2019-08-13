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

//! `ExecutedBlock` is the underlying data structure used by other block types to store block
//! related info. As a block goes through processing we use different types to signal its state:
//! "open", "closed", "locked", "sealed". They all embed an `ExecutedBlock`.

use std::{
	collections::HashSet,
	sync::Arc,
};

use ethereum_types::{H256, U256};

use account_state::State;
use common_types::{
	header::Header,
	receipt::Receipt,
	transaction::SignedTransaction,
};
use state_db::StateDB;
use trace::Tracing;
use vm::{EnvInfo, LastHashes};

/// An internal type for a block's common elements.
#[derive(Clone)]
pub struct ExecutedBlock {
	/// Executed block header.
	pub header: Header,
	/// Executed transactions.
	pub transactions: Vec<SignedTransaction>,
	/// Uncles.
	pub uncles: Vec<Header>,
	/// Transaction receipts.
	pub receipts: Vec<Receipt>,
	/// Hashes of already executed transactions.
	pub transactions_set: HashSet<H256>,
	/// Underlying state.
	pub state: State<StateDB>,
	/// Transaction traces.
	pub traces: Tracing,
	/// Hashes of last 256 blocks.
	pub last_hashes: Arc<LastHashes>,
}

impl ExecutedBlock {
	/// Create a new block from the given `state`.
	pub fn new(state: State<StateDB>, last_hashes: Arc<LastHashes>, tracing: bool) -> ExecutedBlock {
		ExecutedBlock {
			header: Default::default(),
			transactions: Default::default(),
			uncles: Default::default(),
			receipts: Default::default(),
			transactions_set: Default::default(),
			state,
			traces: if tracing {
				Tracing::enabled()
			} else {
				Tracing::Disabled
			},
			last_hashes,
		}
	}

	/// Get the environment info concerning this block.
	pub fn env_info(&self) -> EnvInfo {
		// TODO: memoise.
		EnvInfo {
			number: self.header.number(),
			author: self.header.author().clone(),
			timestamp: self.header.timestamp(),
			difficulty: self.header.difficulty().clone(),
			last_hashes: self.last_hashes.clone(),
			gas_used: self.receipts.last().map_or(U256::zero(), |r| r.gas_used),
			gas_limit: self.header.gas_limit().clone(),
		}
	}

	/// Get mutable access to a state.
	pub fn state_mut(&mut self) -> &mut State<StateDB> {
		&mut self.state
	}

	/// Get mutable reference to traces.
	pub fn traces_mut(&mut self) -> &mut Tracing {
		&mut self.traces
	}
}
