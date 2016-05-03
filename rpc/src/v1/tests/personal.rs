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

use std::sync::Arc;
use jsonrpc_core::IoHandler;
use util::numbers::*;
use util::keys::{TestAccount, TestAccountProvider};
use v1::{PersonalClient, Personal};
use std::collections::*;

fn accounts_provider() -> Arc<TestAccountProvider> {
	let accounts = HashMap::new();
	let ap = TestAccountProvider::new(accounts);
	Arc::new(ap)
}

fn setup() -> (Arc<TestAccountProvider>, IoHandler) {
	let test_provider = accounts_provider();
	let personal = PersonalClient::new(&test_provider);
	let io = IoHandler::new();
	io.add_delegate(personal.to_delegate());
	(test_provider, io)
}

#[test]
fn accounts() {
	let (test_provider, io) = setup();
	test_provider.accounts
		.write()
		.unwrap()
		.insert(Address::from(1), TestAccount::new("test"));

	let request = r#"{"jsonrpc": "2.0", "method": "personal_listAccounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x0000000000000000000000000000000000000001"],"id":1}"#;

	assert_eq!(io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn new_account() {
	let (test_provider, io) = setup();
	let request = r#"{"jsonrpc": "2.0", "method": "personal_newAccount", "params": ["pass"], "id": 1}"#;

	let res = io.handle_request(request);

	let accounts = test_provider.accounts.read().unwrap();
	assert_eq!(accounts.len(), 1);

	let address = accounts
		.keys()
		.nth(0)
		.cloned()
		.unwrap();

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"","id":1}"#;

	assert_eq!(res, Some(response));
}

