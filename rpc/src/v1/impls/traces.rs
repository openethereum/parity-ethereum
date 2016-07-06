// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
use std::collections::BTreeMap;
//use util::H256;
use ethcore::client::{BlockChainClient, CallAnalytics, TransactionID, TraceId};
use ethcore::miner::MinerService;
use ethcore::transaction::{Transaction as EthTransaction, SignedTransaction, Action};
use v1::traits::Traces;
use v1::helpers::CallRequest as CRequest;
use v1::types::{TraceFilter, LocalizedTrace, Trace, BlockNumber, Index, CallRequest, Bytes, StateDiff, VMTrace, H256};

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
				to_value(&traces)
			})
	}

	fn block_traces(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(BlockNumber,)>(params)
			.and_then(|(block_number,)| {
				let client = take_weak!(self.client);
				let traces = client.block_traces(block_number.into());
				let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(LocalizedTrace::from).collect());
				to_value(&traces)
			})
	}

	fn transaction_traces(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(H256,)>(params)
			.and_then(|(transaction_hash,)| {
				let client = take_weak!(self.client);
				let traces = client.transaction_traces(TransactionID::Hash(transaction_hash.into()));
				let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(LocalizedTrace::from).collect());
				to_value(&traces)
			})
	}

	fn trace(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		from_params::<(H256, Vec<Index>)>(params)
			.and_then(|(transaction_hash, address)| {
				let client = take_weak!(self.client);
				let id = TraceId {
					transaction: TransactionID::Hash(transaction_hash.into()),
					address: address.into_iter().map(|i| i.value()).collect()
				};
				let trace = client.trace(id);
				let trace = trace.map(LocalizedTrace::from);
				to_value(&trace)
			})
	}

	fn call(&self, params: Params) -> Result<Value, Error> {
		try!(self.active());
		trace!(target: "jsonrpc", "call: {:?}", params);
		from_params(params)
			.and_then(|(request, flags)| {
				let request = CallRequest::into(request);
				let flags: Vec<String> = flags;
				let analytics = CallAnalytics {
					transaction_tracing: flags.contains(&("trace".to_owned())),
					vm_tracing: flags.contains(&("vmTrace".to_owned())),
					state_diffing: flags.contains(&("stateDiff".to_owned())),
				};
				let signed = try!(self.sign_call(request));
				let r = take_weak!(self.client).call(&signed, analytics);
				if let Ok(executed) = r {
					// TODO maybe add other stuff to this?
					let mut ret = map!["output".to_owned() => to_value(&Bytes(executed.output)).unwrap()];
					if let Some(trace) = executed.trace {
						ret.insert("trace".to_owned(), to_value(&Trace::from(trace)).unwrap());
					}
					if let Some(vm_trace) = executed.vm_trace {
						ret.insert("vmTrace".to_owned(), to_value(&VMTrace::from(vm_trace)).unwrap());
					}
					if let Some(state_diff) = executed.state_diff {
						ret.insert("stateDiff".to_owned(), to_value(&StateDiff::from(state_diff)).unwrap());
					}
					return Ok(Value::Object(ret))
				}
				Ok(Value::Null)
			})
	}
}
