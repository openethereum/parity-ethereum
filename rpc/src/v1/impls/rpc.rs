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

//! RPC generic methods implementation.
use std::collections::BTreeMap;
use jsonrpc_core::*;
use v1::traits::Rpc;

/// RPC generic methods implementation.
pub struct RpcClient {
	modules: BTreeMap<String, String>,
}

impl RpcClient {
	/// Creates new `RpcClient`.
	pub fn new(modules: BTreeMap<String, String>) -> Self {
		RpcClient {
			modules: modules
		}
	}
}

impl Rpc for RpcClient {
	fn modules(&self, _: Params) -> Result<Value, Error> {
		let modules = self.modules.iter().fold(BTreeMap::new(), |mut map, (k, v)| {
			map.insert(k.to_owned(), Value::String(v.to_owned()));
			map
		});
		Ok(Value::Object(modules))
	}
}
