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

//! Traces api implementation.

use jsonrpc_core::Result;
use v1::Metadata;
use v1::traits::Traces;
use v1::helpers::errors;
use v1::types::{TraceFilter, LocalizedTrace, BlockNumber, Index, CallRequest, Bytes, TraceResults, TraceResultsWithTransactionHash, TraceOptions, H256};

/// Traces api implementation.
// TODO: all calling APIs should be possible w. proved remote TX execution.
pub struct TracesClient;

impl Traces for TracesClient {
	type Metadata = Metadata;

	fn filter(&self, _filter: TraceFilter) -> Result<Option<Vec<LocalizedTrace>>> {
		Err(errors::light_unimplemented(None))
	}

	fn block_traces(&self, _block_number: BlockNumber) -> Result<Option<Vec<LocalizedTrace>>> {
		Err(errors::light_unimplemented(None))
	}

	fn transaction_traces(&self, _transaction_hash: H256) -> Result<Option<Vec<LocalizedTrace>>> {
		Err(errors::light_unimplemented(None))
	}

	fn trace(&self, _transaction_hash: H256, _address: Vec<Index>) -> Result<Option<LocalizedTrace>> {
		Err(errors::light_unimplemented(None))
	}

	fn call(&self, _request: CallRequest, _flags: TraceOptions, _block: Option<BlockNumber>) -> Result<TraceResults> {
		Err(errors::light_unimplemented(None))
	}

	fn call_many(&self, _request: Vec<(CallRequest, TraceOptions)>, _block: Option<BlockNumber>) -> Result<Vec<TraceResults>> {
		Err(errors::light_unimplemented(None))
	}

	fn raw_transaction(&self, _raw_transaction: Bytes, _flags: TraceOptions, _block: Option<BlockNumber>) -> Result<TraceResults> {
		Err(errors::light_unimplemented(None))
	}

	fn replay_transaction(&self, _transaction_hash: H256, _flags: TraceOptions) -> Result<TraceResults> {
		Err(errors::light_unimplemented(None))
	}

	fn replay_block_transactions(&self,  _block_number: BlockNumber, _flags: TraceOptions) ->  Result<Vec<TraceResultsWithTransactionHash>> {
		Err(errors::light_unimplemented(None))
	}
}
