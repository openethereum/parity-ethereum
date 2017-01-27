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

//! Traces specific rpc interface.

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use v1::types::{TraceFilter, LocalizedTrace, BlockNumber, Index, CallRequest, Bytes, TraceResults, H256};

build_rpc_trait! {
	/// Traces specific rpc interface.
	pub trait Traces {
		/// Returns traces matching given filter.
		#[rpc(name = "trace_filter")]
		fn filter(&self, TraceFilter) -> Result<Vec<LocalizedTrace>, Error>;

		/// Returns transaction trace at given index.
		#[rpc(name = "trace_get")]
		fn trace(&self, H256, Vec<Index>) -> Result<Option<LocalizedTrace>, Error>;

		/// Returns all traces of given transaction.
		#[rpc(name = "trace_transaction")]
		fn transaction_traces(&self, H256) -> Result<Vec<LocalizedTrace>, Error>;

		/// Returns all traces produced at given block.
		#[rpc(name = "trace_block")]
		fn block_traces(&self, BlockNumber) -> Result<Vec<LocalizedTrace>, Error>;

		/// Executes the given call and returns a number of possible traces for it.
		#[rpc(name = "trace_call")]
		fn call(&self, CallRequest, Vec<String>, Trailing<BlockNumber>) -> Result<Option<TraceResults>, Error>;

		/// Executes the given raw transaction and returns a number of possible traces for it.
		#[rpc(name = "trace_rawTransaction")]
		fn raw_transaction(&self, Bytes, Vec<String>, Trailing<BlockNumber>) -> Result<Option<TraceResults>, Error>;

		/// Executes the transaction with the given hash and returns a number of possible traces for it.
		#[rpc(name = "trace_replayTransaction")]
		fn replay_transaction(&self, H256, Vec<String>) -> Result<Option<TraceResults>, Error>;
	}
}
