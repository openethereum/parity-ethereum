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

use jsonrpc_core::Result;
use jsonrpc_macros::Trailing;
use v1::types::{TraceFilter, LocalizedTrace, BlockNumber, Index, CallRequest, Bytes, TraceResults, H256, TraceOptions};

build_rpc_trait! {
	/// Traces specific rpc interface.
	pub trait Traces {
		type Metadata;

		/// Returns traces matching given filter.
		#[rpc(name = "trace_filter")]
		fn filter(&self, TraceFilter) -> Result<Option<Vec<LocalizedTrace>>>;

		/// Returns transaction trace at given index.
		#[rpc(name = "trace_get")]
		fn trace(&self, H256, Vec<Index>) -> Result<Option<LocalizedTrace>>;

		/// Returns all traces of given transaction.
		#[rpc(name = "trace_transaction")]
		fn transaction_traces(&self, H256) -> Result<Option<Vec<LocalizedTrace>>>;

		/// Returns all traces produced at given block.
		#[rpc(name = "trace_block")]
		fn block_traces(&self, BlockNumber) -> Result<Option<Vec<LocalizedTrace>>>;

		/// Executes the given call and returns a number of possible traces for it.
		#[rpc(meta, name = "trace_call")]
		fn call(&self, Self::Metadata, CallRequest, TraceOptions, Trailing<BlockNumber>) -> Result<TraceResults>;

		/// Executes all given calls and returns a number of possible traces for each of it.
		#[rpc(meta, name = "trace_callMany")]
		fn call_many(&self, Self::Metadata, Vec<(CallRequest, TraceOptions)>, Trailing<BlockNumber>) -> Result<Vec<TraceResults>>;

		/// Executes the given raw transaction and returns a number of possible traces for it.
		#[rpc(name = "trace_rawTransaction")]
		fn raw_transaction(&self, Bytes, TraceOptions, Trailing<BlockNumber>) -> Result<TraceResults>;

		/// Executes the transaction with the given hash and returns a number of possible traces for it.
		#[rpc(name = "trace_replayTransaction")]
		fn replay_transaction(&self, H256, TraceOptions) -> Result<TraceResults>;
	}
}
