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

//! Traces specific rpc interface.

use std::sync::Arc;
use jsonrpc_core::{Params, Value, Error, IoDelegate};

/// Traces specific rpc interface.
pub trait Traces: Sized + Send + Sync + 'static {
	/// Returns traces matching given filter.
	fn filter(&self, _: Params) -> Result<Value, Error>;

	/// Returns transaction trace at given index.
	fn trace(&self, _: Params) -> Result<Value, Error>;

	/// Returns all traces of given transaction.
	fn transaction_traces(&self, _: Params) -> Result<Value, Error>;

	/// Returns all traces produced at given block.
	fn block_traces(&self, _: Params) -> Result<Value, Error>;

	/// Executes the given call and returns a number of possible traces for it.
	fn call(&self, _: Params) -> Result<Value, Error>;

	/// Executes the given raw transaction and returns a number of possible traces for it.
	fn raw_transaction(&self, _: Params) -> Result<Value, Error>;

	/// Executes the transaction with the given hash and returns a number of possible traces for it.
	fn replay_transaction(&self, _: Params) -> Result<Value, Error>;

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("trace_filter", Traces::filter);
		delegate.add_method("trace_get", Traces::trace);
		delegate.add_method("trace_transaction", Traces::transaction_traces);
		delegate.add_method("trace_block", Traces::block_traces);
		delegate.add_method("trace_call", Traces::call);
		delegate.add_method("trace_rawTransaction", Traces::raw_transaction);
		delegate.add_method("trace_replayTransaction", Traces::replay_transaction);

		delegate
	}
}
