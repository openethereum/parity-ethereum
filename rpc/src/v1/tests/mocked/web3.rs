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

use jsonrpc_core::IoHandler;
use util::version;
use v1::{Web3, Web3Client};

#[test]
fn rpc_web3_version() {
	let web3 = Web3Client::new().to_delegate();
	let io = IoHandler::new();
	io.add_delegate(web3);

	let v = version().to_owned().replace("Parity/", "Parity//");

	let request = r#"{"jsonrpc": "2.0", "method": "web3_clientVersion", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"VER","id":1}"#.to_owned().replace("VER", v.as_ref());

	assert_eq!(io.handle_request(request), Some(response));
}

#[test]
fn rpc_web3_sha3() {
	let web3 = Web3Client::new().to_delegate();
	let io = IoHandler::new();
	io.add_delegate(web3);

	let request = r#"{"jsonrpc": "2.0", "method": "web3_sha3", "params": ["0x00"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn rpc_web3_sha3_wiki() {
	let web3 = Web3Client::new().to_delegate();
	let io = IoHandler::new();
	io.add_delegate(web3);

	let request = r#"{"jsonrpc": "2.0", "method": "web3_sha3", "params": ["0x68656c6c6f20776f726c64"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad","id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}
