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

//! Traces api implementation.

use std::sync::Arc;

use account_state::state::StateInfo;
use ethcore::client::Call;
use client_traits::{BlockChainClient, StateClient};
use ethereum_types::H256;
use rlp::Rlp;
use types::{
	call_analytics::CallAnalytics,
	ids::{BlockId, TransactionId, TraceId},
	transaction::SignedTransaction,
};

use jsonrpc_core::Result;
use v1::Metadata;
use v1::traits::Traces;
use v1::helpers::{errors, fake_sign};
use v1::types::{TraceFilter, LocalizedTrace, BlockNumber, Index, CallRequest, Bytes, TraceResults,
	TraceResultsWithTransactionHash, TraceOptions, block_number_to_id};

fn to_call_analytics(flags: TraceOptions) -> CallAnalytics {
	CallAnalytics {
		transaction_tracing: flags.contains(&("trace".to_owned())),
		vm_tracing: flags.contains(&("vmTrace".to_owned())),
		state_diffing: flags.contains(&("stateDiff".to_owned())),
	}
}

/// Traces api implementation.
pub struct TracesClient<C> {
	client: Arc<C>,
}

impl<C> TracesClient<C> {
	/// Creates new Traces client.
	pub fn new(client: &Arc<C>) -> Self {
		TracesClient {
			client: client.clone(),
		}
	}
}

impl<C, S> Traces for TracesClient<C> where
	S: StateInfo + 'static,
	C: BlockChainClient + StateClient<State=S> + Call<State=S> + 'static
{
	type Metadata = Metadata;

	fn filter(&self, filter: TraceFilter) -> Result<Option<Vec<LocalizedTrace>>> {
		Ok(self.client.filter_traces(filter.into())
			.map(|traces| traces.into_iter().map(LocalizedTrace::from).collect()))
	}

	fn block_traces(&self, block_number: BlockNumber) -> Result<Option<Vec<LocalizedTrace>>> {
		let id = match block_number {
			BlockNumber::Pending => return Ok(None),
			num => block_number_to_id(num)
		};

		Ok(self.client.block_traces(id)
			.map(|traces| traces.into_iter().map(LocalizedTrace::from).collect()))
	}

	fn transaction_traces(&self, transaction_hash: H256) -> Result<Option<Vec<LocalizedTrace>>> {
		Ok(self.client.transaction_traces(TransactionId::Hash(transaction_hash))
			.map(|traces| traces.into_iter().map(LocalizedTrace::from).collect()))
	}

	fn trace(&self, transaction_hash: H256, address: Vec<Index>) -> Result<Option<LocalizedTrace>> {
		let id = TraceId {
			transaction: TransactionId::Hash(transaction_hash),
			address: address.into_iter().map(|i| i.value()).collect()
		};

		Ok(self.client.trace(id)
			.map(LocalizedTrace::from))
	}

	fn call(&self, request: CallRequest, flags: TraceOptions, block: Option<BlockNumber>) -> Result<TraceResults> {
		let block = block.unwrap_or_default();

		let request = CallRequest::into(request);
		let signed = fake_sign::sign_call(request)?;

		let id = match block {
			BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
			BlockNumber::Num(num) => BlockId::Number(num),
			BlockNumber::Earliest => BlockId::Earliest,
			BlockNumber::Latest => BlockId::Latest,

			BlockNumber::Pending => return Err(errors::invalid_params("`BlockNumber::Pending` is not supported", ())),
		};

		let mut state = self.client.state_at(id).ok_or_else(errors::state_pruned)?;
		let header = self.client.block_header(id).ok_or_else(errors::state_pruned)?;

		self.client.call(&signed, to_call_analytics(flags), &mut state, &header.decode().map_err(errors::decode)?)
			.map(TraceResults::from)
			.map_err(errors::call)
	}

	fn call_many(&self, requests: Vec<(CallRequest, TraceOptions)>, block: Option<BlockNumber>) -> Result<Vec<TraceResults>> {
		let block = block.unwrap_or_default();

		let requests = requests.into_iter()
			.map(|(request, flags)| {
				let request = CallRequest::into(request);
				let signed = fake_sign::sign_call(request)?;
				Ok((signed, to_call_analytics(flags)))
			})
			.collect::<Result<Vec<_>>>()?;

		let id = match block {
			BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
			BlockNumber::Num(num) => BlockId::Number(num),
			BlockNumber::Earliest => BlockId::Earliest,
			BlockNumber::Latest => BlockId::Latest,

			BlockNumber::Pending => return Err(errors::invalid_params("`BlockNumber::Pending` is not supported", ())),
		};

		let mut state = self.client.state_at(id).ok_or_else(errors::state_pruned)?;
		let header = self.client.block_header(id).ok_or_else(errors::state_pruned)?;

		self.client.call_many(&requests, &mut state, &header.decode().map_err(errors::decode)?)
			.map(|results| results.into_iter().map(TraceResults::from).collect())
			.map_err(errors::call)
	}

	fn raw_transaction(&self, raw_transaction: Bytes, flags: TraceOptions, block: Option<BlockNumber>) -> Result<TraceResults> {
		let block = block.unwrap_or_default();

		let tx = Rlp::new(&raw_transaction.into_vec()).as_val().map_err(|e| errors::invalid_params("Transaction is not valid RLP", e))?;
		let signed = SignedTransaction::new(tx).map_err(errors::transaction)?;

		let id = match block {
			BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
			BlockNumber::Num(num) => BlockId::Number(num),
			BlockNumber::Earliest => BlockId::Earliest,
			BlockNumber::Latest => BlockId::Latest,

			BlockNumber::Pending => return Err(errors::invalid_params("`BlockNumber::Pending` is not supported", ())),
		};

		let mut state = self.client.state_at(id).ok_or_else(errors::state_pruned)?;
		let header = self.client.block_header(id).ok_or_else(errors::state_pruned)?;

		self.client.call(&signed, to_call_analytics(flags), &mut state, &header.decode().map_err(errors::decode)?)
			.map(TraceResults::from)
			.map_err(errors::call)
	}

	fn replay_transaction(&self, transaction_hash: H256, flags: TraceOptions) -> Result<TraceResults> {
		self.client.replay(TransactionId::Hash(transaction_hash), to_call_analytics(flags))
			.map(TraceResults::from)
			.map_err(errors::call)
	}

	fn replay_block_transactions(&self, block_number: BlockNumber, flags: TraceOptions) -> Result<Vec<TraceResultsWithTransactionHash>> {
		let id = match block_number {
			BlockNumber::Hash { hash, .. } => BlockId::Hash(hash),
			BlockNumber::Num(num) => BlockId::Number(num),
			BlockNumber::Earliest => BlockId::Earliest,
			BlockNumber::Latest => BlockId::Latest,

			BlockNumber::Pending => return Err(errors::invalid_params("`BlockNumber::Pending` is not supported", ())),
		};

		self.client.replay_block_transactions(id, to_call_analytics(flags))
			.map(|results| results.map(TraceResultsWithTransactionHash::from).collect())
			.map_err(errors::call)
	}
}
