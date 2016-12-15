// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
use jsonrpc_core::*;
use serde;

use rlp::{UntrustedRlp, View};
use ethcore::client::{BlockChainClient, CallAnalytics, TransactionId, TraceId};
use ethcore::miner::MinerService;
use ethcore::transaction::{Transaction as EthTransaction, SignedTransaction, Action};

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

/// Returns number of different parameters in given `Params` object.
fn params_len(params: &Params) -> usize {
	match params {
		&Params::Array(ref vec) => vec.len(),
		_ => 0,
	}
}

/// Deserialize request parameters with optional third parameter `BlockNumber` defaulting to `BlockNumber::Latest`.
fn from_params_default_third<F1, F2>(params: Params) -> Result<(F1, F2, BlockNumber, ), Error> where F1: serde::de::Deserialize, F2: serde::de::Deserialize {
	match params_len(&params) {
		2 => from_params::<(F1, F2, )>(params).map(|(f1, f2)| (f1, f2, BlockNumber::Latest)),
		_ => from_params::<(F1, F2, BlockNumber)>(params)
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

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C, M> Traces for TracesClient<C, M> where C: BlockChainClient + 'static, M: MinerService + 'static {
	fn filter(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(TraceFilter,)>(params)
			.and_then(|(filter, )| {
				let client = take_weak!(self.client);
				let traces = client.filter_traces(filter.into());
				let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(LocalizedTrace::from).collect());
				Ok(to_value(&traces))
			})
	}

	fn block_traces(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(BlockNumber,)>(params)
			.and_then(|(block_number,)| {
				let client = take_weak!(self.client);
				let traces = client.block_traces(block_number.into());
				let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(LocalizedTrace::from).collect());
				Ok(to_value(&traces))
			})
	}

	fn transaction_traces(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(H256,)>(params)
			.and_then(|(transaction_hash,)| {
				let client = take_weak!(self.client);
				let traces = client.transaction_traces(TransactionId::Hash(transaction_hash.into()));
				let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(LocalizedTrace::from).collect());
				Ok(to_value(&traces))
			})
	}

	fn trace(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(H256, Vec<Index>)>(params)
			.and_then(|(transaction_hash, address)| {
				let client = take_weak!(self.client);
				let id = TraceId {
					transaction: TransactionId::Hash(transaction_hash.into()),
					address: address.into_iter().map(|i| i.value()).collect()
				};
				let trace = client.trace(id);
				let trace = trace.map(LocalizedTrace::from);
				Ok(to_value(&trace))
			})
	}

	fn call(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params_default_third(params)
			.and_then(|(request, flags, block)| {
				let request = CallRequest::into(request);
				let signed = try!(self.sign_call(request));
				match take_weak!(self.client).call(&signed, block.into(), to_call_analytics(flags)) {
					Ok(e) => Ok(to_value(&TraceResults::from(e))),
					_ => Ok(Value::Null),
				}
			})
	}

	fn raw_transaction(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params_default_third(params)
			.and_then(|(raw_transaction, flags, block)| {
				let raw_transaction = Bytes::to_vec(raw_transaction);
				match UntrustedRlp::new(&raw_transaction).as_val() {
					Ok(signed) => match take_weak!(self.client).call(&signed, block.into(), to_call_analytics(flags)) {
						Ok(e) => Ok(to_value(&TraceResults::from(e))),
						_ => Ok(Value::Null),
					},
					Err(e) => Err(errors::invalid_params("Transaction is not valid RLP", e)),
				}
			})
	}

	fn replay_transaction(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(H256, _)>(params)
			.and_then(|(transaction_hash, flags)| {
				match take_weak!(self.client).replay(TransactionId::Hash(transaction_hash.into()), to_call_analytics(flags)) {
					Ok(e) => Ok(to_value(&TraceResults::from(e))),
					_ => Ok(Value::Null),
				}
			})
	}
}
