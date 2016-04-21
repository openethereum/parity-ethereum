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
use ethcore::client::BlockChainClient;
use v1::traits::Traces;
use v1::types::{TraceFilter, Trace};

/// Traces api implementation.
pub struct TracesClient<C> where C: BlockChainClient {
	client: Weak<C>,
}

impl<C> TracesClient<C> where C: BlockChainClient {
	/// Creates new Traces client.
	pub fn new(client: &Arc<C>) -> Self {
		TracesClient {
			client: Arc::downgrade(client),
		}
	}
}

impl<C> Traces for TracesClient<C> where C: BlockChainClient + 'static {
	fn filter(&self, params: Params) -> Result<Value, Error> {
		from_params::<(TraceFilter,)>(params)
			.and_then(|(filter, )| {
				let client = take_weak!(self.client);
				let traces = client.filter_traces(filter.into());
				let traces = traces.map_or_else(Vec::new, |traces| traces.into_iter().map(Trace::from).collect());
				to_value(&traces)
			})
	}
}
