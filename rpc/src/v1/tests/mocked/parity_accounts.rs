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

use ethcore::account_provider::AccountProvider;
use ethcore::client::TestBlockChainClient;

use jsonrpc_core::IoHandler;
use v1::{ParityAccounts, ParityAccountsClient};

struct ParityAccountsTester {
	accounts: Arc<AccountProvider>,
	io: IoHandler,
	// these unused fields are necessary to keep the data alive
	// as the handler has only weak pointers.
	_client: Arc<TestBlockChainClient>,
}

fn blockchain_client() -> Arc<TestBlockChainClient> {
	let client = TestBlockChainClient::new();
	Arc::new(client)
}

fn accounts_provider() -> Arc<AccountProvider> {
	Arc::new(AccountProvider::transient_provider())
}

fn setup() -> ParityAccountsTester {
	let accounts = accounts_provider();
	let client = blockchain_client();
	let parity_accounts = ParityAccountsClient::new(&accounts, &client);

	let io = IoHandler::new();
	io.add_delegate(parity_accounts.to_delegate());

	let tester = ParityAccountsTester {
		accounts: accounts,
		io: io,
		_client: client,
	};

	tester
}

#[test]
fn should_be_able_to_get_account_info() {
	let tester = setup();
	tester.accounts.new_account("").unwrap();
	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	let uuid = tester.accounts.accounts_info().unwrap().get(&address).unwrap().uuid.as_ref().unwrap().clone();
	tester.accounts.set_account_name(address.clone(), "Test".to_owned()).unwrap();
	tester.accounts.set_account_meta(address.clone(), "{foo: 69}".to_owned()).unwrap();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_accountsInfo", "params": [], "id": 1}"#;
	let res = tester.io.handle_request_sync(request);
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"0x{}\":{{\"meta\":\"{{foo: 69}}\",\"name\":\"Test\",\"uuid\":\"{}\"}}}},\"id\":1}}", address.hex(), uuid);
	assert_eq!(res, Some(response));
}

#[test]
fn should_be_able_to_set_name() {
	let tester = setup();
	tester.accounts.new_account("").unwrap();
	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_setAccountName", "params": ["0x{}", "Test"], "id": 1}}"#, address.hex());
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	let uuid = tester.accounts.accounts_info().unwrap().get(&address).unwrap().uuid.as_ref().unwrap().clone();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_accountsInfo", "params": [], "id": 1}"#;
	let res = tester.io.handle_request_sync(request);
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"0x{}\":{{\"meta\":\"{{}}\",\"name\":\"Test\",\"uuid\":\"{}\"}}}},\"id\":1}}", address.hex(), uuid);
	assert_eq!(res, Some(response));
}

#[test]
fn should_be_able_to_set_meta() {
	let tester = setup();
	tester.accounts.new_account("").unwrap();
	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_setAccountMeta", "params": ["0x{}", "{{foo: 69}}"], "id": 1}}"#, address.hex());
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	let uuid = tester.accounts.accounts_info().unwrap().get(&address).unwrap().uuid.as_ref().unwrap().clone();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_accountsInfo", "params": [], "id": 1}"#;
	let res = tester.io.handle_request_sync(request);
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"0x{}\":{{\"meta\":\"{{foo: 69}}\",\"name\":\"\",\"uuid\":\"{}\"}}}},\"id\":1}}", address.hex(), uuid);
	assert_eq!(res, Some(response));
}

#[test]
fn should_be_able_to_kill_account() {
	let tester = setup();
	tester.accounts.new_account("password").unwrap();
	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_killAccount", "params": ["0xf00baba2f00baba2f00baba2f00baba2f00baba2"], "id": 1}}"#);
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params","data":null},"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_killAccount", "params": ["0x{}", "password"], "id": 1}}"#, address.hex());
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 0);
}

