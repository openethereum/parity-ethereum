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

//! Traces api implementation.

use std::sync::{Weak, Arc};

use rlp::{UntrustedRlp, View};
use ethcore::client::{BlockChainClient, CallAnalytics, TransactionId, TraceId};
use ethcore::miner::MinerService;
use ethcore::transaction::{Transaction as EthTransaction, SignedTransaction, Action};

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use v1::traits::Traces;
use v1::helpers::{errors, CallRequest as CRequest};
use v1::types::{TraceFilter, LocalizedTrace, BlockNumber, Index, CallRequest, Bytes, TraceResults, H256};

fn to_call_analytics(flags: Vec<String>) -> CallAnalytics {
	CallAnalytics {
		transaction_tracing: flags.contains(&("trace".to_owned())),
		vm_tracing: flags.contains(&("vmTrace".to_owned())),
		state_diffing: flags.contains(&("stateDiff".to_owned())),
	}
}

/// Traces api implementation.
pub struct TracesClient<C, M> where C: BlockChainClient, M: MinerService {
	client: Weak<C>,
	miner: Weak<M>,
}

impl<C, M> TracesClient<C, M> where C: BlockChainClient, M: MinerService {
	/// Creates new Traces client.
	pub fn new(client: &Arc<C>, miner: &Arc<M>) -> Self {
		TracesClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
		}
	}

	// TODO: share with eth.rs
	fn sign_call(&self, request: CRequest) -> Result<SignedTransaction, Error> {
		let client = take_weak!(self.client);
		let miner = take_weak!(self.miner);
		let from = request.from.unwrap_or(0.into());
		Ok(EthTransaction {
			nonce: request.nonce.unwrap_or_else(|| client.latest_nonce(&from)),
			action: request.to.map_or(Action::Create, Action::Call),
			gas: request.gas.unwrap_or(50_000_000.into()),
			gas_price: request.gas_price.unwrap_or_else(|| miner.sensible_gas_price()),
			value: request.value.unwrap_or(0.into()),
			data: request.data.map_or_else(Vec::new, |d| d.to_vec())
		}.fake_sign(from))
	}
}

impl<C, M> Traces for TracesClient<C, M> where C: BlockChainClient + 'static, M: MinerService + 'static {
	fn filter(&self, filter: TraceFilter) -> Result<Vec<LocalizedTrace>, Error> {
		let client = take_weak!(self.client);
		let traces = client.filter_traces(filter.into());
		let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(LocalizedTrace::from).collect());
		Ok(traces)
	}

	fn block_traces(&self, block_number: BlockNumber) -> Result<Vec<LocalizedTrace>, Error> {
		let client = take_weak!(self.client);
		let traces = client.block_traces(block_number.into());
		let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(LocalizedTrace::from).collect());
		Ok(traces)
	}

	fn transaction_traces(&self, transaction_hash: H256) -> Result<Vec<LocalizedTrace>, Error> {
		let client = take_weak!(self.client);
		let traces = client.transaction_traces(TransactionId::Hash(transaction_hash.into()));
		let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(LocalizedTrace::from).collect());
		Ok(traces)
	}

	fn trace(&self, transaction_hash: H256, address: Vec<Index>) -> Result<Option<LocalizedTrace>, Error> {
		let client = take_weak!(self.client);
		let id = TraceId {
			transaction: TransactionId::Hash(transaction_hash.into()),
			address: address.into_iter().map(|i| i.value()).collect()
		};
		let trace = client.trace(id);
		let trace = trace.map(LocalizedTrace::from);

		Ok(trace)
	}

	fn call(&self, request: CallRequest, flags: Vec<String>, block: Trailing<BlockNumber>) -> Result<Option<TraceResults>, Error> {
		let block = block.0;

		let request = CallRequest::into(request);
		let signed = self.sign_call(request)?;
		Ok(match take_weak!(self.client).call(&signed, block.into(), to_call_analytics(flags)) {
			Ok(e) => Some(TraceResults::from(e)),
			_ => None,
		})
	}

	fn raw_transaction(&self, raw_transaction: Bytes, flags: Vec<String>, block: Trailing<BlockNumber>) -> Result<Option<TraceResults>, Error> {
		let block = block.0;

		UntrustedRlp::new(&raw_transaction.into_vec()).as_val()
			.map_err(|e| errors::invalid_params("Transaction is not valid RLP", e))
			.and_then(|tx| SignedTransaction::new(tx).map_err(errors::from_transaction_error))
			.and_then(|signed| {
				Ok(match take_weak!(self.client).call(&signed, block.into(), to_call_analytics(flags)) {
					Ok(e) => Some(TraceResults::from(e)),
					_ => None,
				})
			})
	}

	fn replay_transaction(&self, transaction_hash: H256, flags: Vec<String>) -> Result<Option<TraceResults>, Error> {
		Ok(match take_weak!(self.client).replay(TransactionId::Hash(transaction_hash.into()), to_call_analytics(flags)) {
			Ok(e) => Some(TraceResults::from(e)),
			_ => None,
		})
	}
}
