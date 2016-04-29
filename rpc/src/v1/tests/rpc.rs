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

use std::collections::BTreeMap;
use jsonrpc_core::IoHandler;
use v1::{Rpc, RpcClient};


fn rpc_client() -> RpcClient {
	let mut modules = BTreeMap::new();
	modules.insert("rpc".to_owned(), "1.0".to_owned());
	RpcClient::new(modules)
}

#[test]
fn rpc_modules() {
	let rpc = rpc_client().to_delegate();
	let io = IoHandler::new();
	io.add_delegate(rpc);

	let request = r#"{"jsonrpc": "2.0", "method": "modules", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"eth": "1.0"},"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}
