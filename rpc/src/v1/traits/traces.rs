// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Traces specific rpc interface.

use ethereum_types::H256;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use v1::types::{TraceFilter, LocalizedTrace, BlockNumber, Index, CallRequest, Bytes, TraceResults,
	TraceResultsWithTransactionHash, TraceOptions};

/// Traces specific rpc interface.
#[rpc(server)]
pub trait Traces {
	/// RPC Metadata
	type Metadata;

	/// Returns traces matching given filter.
	#[rpc(name = "trace_filter")]
	fn filter(&self, _: TraceFilter) -> Result<Option<Vec<LocalizedTrace>>>;

	/// Returns transaction trace at given index.
	#[rpc(name = "trace_get")]
	fn trace(&self, _: H256, _: Vec<Index>) -> Result<Option<LocalizedTrace>>;

	/// Returns all traces of given transaction.
	#[rpc(name = "trace_transaction")]
	fn transaction_traces(&self, _: H256) -> Result<Option<Vec<LocalizedTrace>>>;

	/// Returns all traces produced at given block.
	#[rpc(name = "trace_block")]
	fn block_traces(&self, _: BlockNumber) -> Result<Option<Vec<LocalizedTrace>>>;

	/// Executes the given call and returns a number of possible traces for it.
	#[rpc(name = "trace_call")]
	fn call(&self, _: CallRequest, _: TraceOptions, _: Option<BlockNumber>) -> Result<TraceResults>;

	/// Executes all given calls and returns a number of possible traces for each of it.
	#[rpc(name = "trace_callMany")]
	fn call_many(&self, _: Vec<(CallRequest, TraceOptions)>, _: Option<BlockNumber>) -> Result<Vec<TraceResults>>;

	/// Executes the given raw transaction and returns a number of possible traces for it.
	#[rpc(name = "trace_rawTransaction")]
	fn raw_transaction(&self, _: Bytes, _: TraceOptions, _: Option<BlockNumber>) -> Result<TraceResults>;

	/// Executes the transaction with the given hash and returns a number of possible traces for it.
	#[rpc(name = "trace_replayTransaction")]
	fn replay_transaction(&self, _: H256, _: TraceOptions) -> Result<TraceResults>;

	/// Executes all the transactions at the given block and returns a number of possible traces for each transaction.
	#[rpc(name = "trace_replayBlockTransactions")]
	fn replay_block_transactions(
		&self,
		_: BlockNumber,
		_: TraceOptions
	) -> Result<Vec<TraceResultsWithTransactionHash>>;
}
