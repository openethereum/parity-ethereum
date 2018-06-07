// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use ethcore::account_provider::{AccountProvider, AccountProviderSettings};
use ethstore::EthStore;
use ethstore::accounts_dir::RootDiskDirectory;
use tempdir::TempDir;

use jsonrpc_core::IoHandler;
use v1::{ParityAccounts, ParityAccountsClient};

struct ParityAccountsTester {
	accounts: Arc<AccountProvider>,
	io: IoHandler,
}

fn accounts_provider() -> Arc<AccountProvider> {
	Arc::new(AccountProvider::transient_provider())
}

fn accounts_provider_with_vaults_support(temp_path: &str) -> Arc<AccountProvider> {
	let root_keys_dir = RootDiskDirectory::create(temp_path).unwrap();
	let secret_store = EthStore::open(Box::new(root_keys_dir)).unwrap();
	Arc::new(AccountProvider::new(Box::new(secret_store), AccountProviderSettings::default()))
}

fn setup_with_accounts_provider(accounts_provider: Arc<AccountProvider>) -> ParityAccountsTester {
	let opt_ap = accounts_provider.clone();
	let parity_accounts = ParityAccountsClient::new(&opt_ap);
	let mut io = IoHandler::default();
	io.extend_with(parity_accounts.to_delegate());

	let tester = ParityAccountsTester {
		accounts: accounts_provider,
		io: io,
	};

	tester
}

fn setup() -> ParityAccountsTester {
	setup_with_accounts_provider(accounts_provider())
}

fn setup_with_vaults_support(temp_path: &str) -> ParityAccountsTester {
	setup_with_accounts_provider(accounts_provider_with_vaults_support(temp_path))
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

	let request = r#"{"jsonrpc": "2.0", "method": "parity_allAccountsInfo", "params": [], "id": 1}"#;
	let res = tester.io.handle_request_sync(request);
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"0x{:x}\":{{\"meta\":\"{{foo: 69}}\",\"name\":\"Test\",\"uuid\":\"{}\"}}}},\"id\":1}}", address, uuid);
	assert_eq!(res, Some(response));
}

#[test]
fn should_be_able_to_set_name() {
	let tester = setup();
	tester.accounts.new_account("").unwrap();
	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_setAccountName", "params": ["0x{:x}", "Test"], "id": 1}}"#, address);
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	let uuid = tester.accounts.accounts_info().unwrap().get(&address).unwrap().uuid.as_ref().unwrap().clone();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_allAccountsInfo", "params": [], "id": 1}"#;
	let res = tester.io.handle_request_sync(request);
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"0x{:x}\":{{\"meta\":\"{{}}\",\"name\":\"Test\",\"uuid\":\"{}\"}}}},\"id\":1}}", address, uuid);
	assert_eq!(res, Some(response));
}

#[test]
fn should_be_able_to_set_meta() {
	let tester = setup();
	tester.accounts.new_account("").unwrap();
	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_setAccountMeta", "params": ["0x{:x}", "{{foo: 69}}"], "id": 1}}"#, address);
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	let uuid = tester.accounts.accounts_info().unwrap().get(&address).unwrap().uuid.as_ref().unwrap().clone();

	let request = r#"{"jsonrpc": "2.0", "method": "parity_allAccountsInfo", "params": [], "id": 1}"#;
	let res = tester.io.handle_request_sync(request);
	let response = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"0x{:x}\":{{\"meta\":\"{{foo: 69}}\",\"name\":\"\",\"uuid\":\"{}\"}}}},\"id\":1}}", address, uuid);
	assert_eq!(res, Some(response));
}

#[test]
fn rpc_parity_set_and_get_dapps_accounts() {
	// given
	let tester = setup();
	tester.accounts.set_address_name(10.into(), "10".into());
	assert_eq!(tester.accounts.dapp_addresses("app1".into()).unwrap(), vec![]);

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "parity_setDappAddresses","params":["app1",["0x000000000000000000000000000000000000000a","0x0000000000000000000000000000000000000001"]], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// then
	assert_eq!(tester.accounts.dapp_addresses("app1".into()).unwrap(), vec![10.into()]);
	let request = r#"{"jsonrpc": "2.0", "method": "parity_getDappAddresses","params":["app1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x000000000000000000000000000000000000000a"],"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_set_and_get_dapp_default_address() {
	// given
	let tester = setup();
	tester.accounts.set_address_name(10.into(), "10".into());
	assert_eq!(tester.accounts.dapp_addresses("app1".into()).unwrap(), vec![]);

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "parity_setDappDefaultAddress","params":["app1", "0x000000000000000000000000000000000000000a"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// then
	assert_eq!(tester.accounts.dapp_addresses("app1".into()).unwrap(), vec![10.into()]);
	let request = r#"{"jsonrpc": "2.0", "method": "parity_getDappDefaultAddress","params":["app1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x000000000000000000000000000000000000000a","id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_set_and_get_new_dapps_whitelist() {
	// given
	let tester = setup();

	// when set to whitelist
	let request = r#"{"jsonrpc": "2.0", "method": "parity_setNewDappsAddresses","params":[["0x000000000000000000000000000000000000000a"]], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// then
	assert_eq!(tester.accounts.new_dapps_addresses().unwrap(), Some(vec![10.into()]));
	let request = r#"{"jsonrpc": "2.0", "method": "parity_getNewDappsAddresses","params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":["0x000000000000000000000000000000000000000a"],"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// when set to empty
	let request = r#"{"jsonrpc": "2.0", "method": "parity_setNewDappsAddresses","params":[null], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// then
	assert_eq!(tester.accounts.new_dapps_addresses().unwrap(), None);
	let request = r#"{"jsonrpc": "2.0", "method": "parity_getNewDappsAddresses","params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_set_and_get_new_dapps_default_address() {
	// given
	let tester = setup();
	tester.accounts.set_address_name(10.into(), "10".into());
	assert_eq!(tester.accounts.new_dapps_default_address().unwrap(), 0.into());

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "parity_setNewDappsDefaultAddress","params":["0x000000000000000000000000000000000000000a"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// then
	assert_eq!(tester.accounts.new_dapps_default_address().unwrap(), 10.into());
	let request = r#"{"jsonrpc": "2.0", "method": "parity_getNewDappsDefaultAddress","params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x000000000000000000000000000000000000000a","id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_recent_dapps() {
	// given
	let tester = setup();

	// when
	// trigger dapp usage
	tester.accounts.note_dapp_used("dapp1".into()).unwrap();

	// then
	let request = r#"{"jsonrpc": "2.0", "method": "parity_listRecentDapps","params":[], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"dapp1":1},"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn should_be_able_to_kill_account() {
	let tester = setup();
	tester.accounts.new_account("password").unwrap();
	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_killAccount", "params": ["0xf00baba2f00baba2f00baba2f00baba2f00baba2"], "id": 1}}"#);
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: invalid length 1, expected a tuple of size 2."},"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_killAccount", "params": ["0x{:x}", "password"], "id": 1}}"#, address);
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 0);
}

#[test]
fn should_be_able_to_remove_address() {
	let tester = setup();

	// add an address
	let request = r#"{"jsonrpc": "2.0", "method": "parity_setAccountName", "params": ["0x000baba1000baba2000baba3000baba4000baba5", "Test"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	// verify it exists
	let request = r#"{"jsonrpc": "2.0", "method": "parity_allAccountsInfo", "params": [], "id": 2}"#;
	let res = tester.io.handle_request_sync(request);
	let response = r#"{"jsonrpc":"2.0","result":{"0x000baba1000baba2000baba3000baba4000baba5":{"meta":"{}","name":"Test"}},"id":2}"#;
	assert_eq!(res, Some(response.into()));

	// remove the address
	let request = r#"{"jsonrpc": "2.0", "method": "parity_removeAddress", "params": ["0x000baba1000baba2000baba3000baba4000baba5"], "id": 3}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":3}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	// verify empty
	let request = r#"{"jsonrpc": "2.0", "method": "parity_allAccountsInfo", "params": [], "id": 4}"#;
	let res = tester.io.handle_request_sync(request);
	let response = r#"{"jsonrpc":"2.0","result":{},"id":4}"#;
	assert_eq!(res, Some(response.into()));
}

#[test]
fn rpc_parity_new_vault() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_newVault", "params":["vault1", "password1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
	assert!(tester.accounts.close_vault("vault1").is_ok());
	assert!(tester.accounts.open_vault("vault1", "password1").is_ok());
}

#[test]
fn rpc_parity_open_vault() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	assert!(tester.accounts.create_vault("vault1", "password1").is_ok());
	assert!(tester.accounts.close_vault("vault1").is_ok());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_openVault", "params":["vault1", "password1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_close_vault() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	assert!(tester.accounts.create_vault("vault1", "password1").is_ok());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_closeVault", "params":["vault1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_change_vault_password() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	assert!(tester.accounts.create_vault("vault1", "password1").is_ok());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_changeVaultPassword", "params":["vault1", "password2"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_change_vault() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	let (address, _) = tester.accounts.new_account_and_public("root_password").unwrap();
	assert!(tester.accounts.create_vault("vault1", "password1").is_ok());

	let request = format!(r#"{{"jsonrpc": "2.0", "method": "parity_changeVault", "params":["0x{:x}", "vault1"], "id": 1}}"#, address);
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(&request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_vault_adds_vault_field_to_acount_meta() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	let (address1, _) = tester.accounts.new_account_and_public("root_password1").unwrap();
	let uuid1 = tester.accounts.account_meta(address1.clone()).unwrap().uuid.unwrap();
	assert!(tester.accounts.create_vault("vault1", "password1").is_ok());
	assert!(tester.accounts.change_vault(address1, "vault1").is_ok());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_allAccountsInfo", "params":[], "id": 1}"#;
	let response = format!(r#"{{"jsonrpc":"2.0","result":{{"0x{:x}":{{"meta":"{{\"vault\":\"vault1\"}}","name":"","uuid":"{}"}}}},"id":1}}"#, address1, uuid1);

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// and then
	assert!(tester.accounts.change_vault(address1, "").is_ok());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_allAccountsInfo", "params":[], "id": 1}"#;
	let response = format!(r#"{{"jsonrpc":"2.0","result":{{"0x{:x}":{{"meta":"{{}}","name":"","uuid":"{}"}}}},"id":1}}"#, address1, uuid1);

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_parity_list_vaults() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	assert!(tester.accounts.create_vault("vault1", "password1").is_ok());
	assert!(tester.accounts.create_vault("vault2", "password2").is_ok());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_listVaults", "params":[], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":["vault1","vault2"],"id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","result":["vault2","vault1"],"id":1}"#;

	let actual_response = tester.io.handle_request_sync(request);
	assert!(actual_response == Some(response1.to_owned())
		|| actual_response == Some(response2.to_owned()));
}

#[test]
fn rpc_parity_list_opened_vaults() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	assert!(tester.accounts.create_vault("vault1", "password1").is_ok());
	assert!(tester.accounts.create_vault("vault2", "password2").is_ok());
	assert!(tester.accounts.create_vault("vault3", "password3").is_ok());
	assert!(tester.accounts.close_vault("vault2").is_ok());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_listOpenedVaults", "params":[], "id": 1}"#;
	let response1 = r#"{"jsonrpc":"2.0","result":["vault1","vault3"],"id":1}"#;
	let response2 = r#"{"jsonrpc":"2.0","result":["vault3","vault1"],"id":1}"#;

	let actual_response = tester.io.handle_request_sync(request);
	assert!(actual_response == Some(response1.to_owned())
		|| actual_response == Some(response2.to_owned()));
}

#[test]
fn rpc_parity_get_set_vault_meta() {
	let tempdir = TempDir::new("").unwrap();
	let tester = setup_with_vaults_support(tempdir.path().to_str().unwrap());

	assert!(tester.accounts.create_vault("vault1", "password1").is_ok());

	// when no meta set
	let request = r#"{"jsonrpc": "2.0", "method": "parity_getVaultMeta", "params":["vault1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"{}","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// when meta set
	assert!(tester.accounts.set_vault_meta("vault1", "vault1_meta").is_ok());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_getVaultMeta", "params":["vault1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"vault1_meta","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// change meta
	let request = r#"{"jsonrpc": "2.0", "method": "parity_setVaultMeta", "params":["vault1", "updated_vault1_meta"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));

	// query changed meta
	let request = r#"{"jsonrpc": "2.0", "method": "parity_getVaultMeta", "params":["vault1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"updated_vault1_meta","id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

// name: parity_deriveAddressHash
// example: {"jsonrpc": "2.0", "method": "parity_deriveAddressHash", "params": ["0xc171033d5cbff7175f29dfd3a63dda3d6f8f385e", "password1", { "type": "soft", "hash": "0x0c0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0c0c" }, true ], "id": 3}
#[test]
fn derive_key_hash() {
	let tester = setup();
	let hash = tester.accounts
		.insert_account(
			"0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a".parse().unwrap(),
			"password1")
		.expect("account should be inserted ok");

	assert_eq!(hash, "c171033d5cbff7175f29dfd3a63dda3d6f8f385e".parse().unwrap());

	// derive by hash
	let request = r#"{"jsonrpc": "2.0", "method": "parity_deriveAddressHash", "params": ["0xc171033d5cbff7175f29dfd3a63dda3d6f8f385e", "password1", { "type": "soft", "hash": "0x0c0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0c0c" }, true ], "id": 3}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xf28c28fcddf4a9b8f474237278d3647f9c0d1b3c","id":3}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));
}

// name: parity_deriveAddressIndex
// example: {"jsonrpc": "2.0", "method": "parity_deriveAddressIndex", "params": ["0xc171033d5cbff7175f29dfd3a63dda3d6f8f385e", "password1", [{ "type": "soft", "index": 0 }, { "type": "soft", "index": 1 }], false ], "id": 3}
#[test]
fn derive_key_index() {
	let tester = setup();
	let hash = tester.accounts
		.insert_account(
			"0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a".parse().unwrap(),
			"password1")
		.expect("account should be inserted ok");

	assert_eq!(hash, "c171033d5cbff7175f29dfd3a63dda3d6f8f385e".parse().unwrap());

	// derive by hash
	let request = r#"{"jsonrpc": "2.0", "method": "parity_deriveAddressIndex", "params": ["0xc171033d5cbff7175f29dfd3a63dda3d6f8f385e", "password1", [{ "type": "soft", "index": 0 }, { "type": "soft", "index": 1 }], false ], "id": 3}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0xcc548e0bb2efe792a920ae0fbf583b13919f274f","id":3}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));
}

#[test]
fn should_export_account() {
	// given
	let tester = setup();
	let wallet = r#"{"id":"6a186c80-7797-cff2-bc2e-7c1d6a6cc76e","version":3,"crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"a1c6ff99070f8032ca1c4e8add006373"},"ciphertext":"df27e3db64aa18d984b6439443f73660643c2d119a6f0fa2fa9a6456fc802d75","kdf":"pbkdf2","kdfparams":{"c":10240,"dklen":32,"prf":"hmac-sha256","salt":"ddc325335cda5567a1719313e73b4842511f3e4a837c9658eeb78e51ebe8c815"},"mac":"3dc888ae79cbb226ff9c455669f6cf2d79be72120f2298f6cb0d444fddc0aa3d"},"address":"0042e5d2a662eeaca8a7e828c174f98f35d8925b","name":"parity-export-test","meta":"{\"passwordHint\":\"parity-export-test\",\"timestamp\":1490017814987}"}"#;
	tester.accounts.import_wallet(wallet.as_bytes(), "parity-export-test", false).unwrap();
	let accounts = tester.accounts.accounts().unwrap();
	assert_eq!(accounts.len(), 1);

	// invalid password
	let request = r#"{"jsonrpc":"2.0","method":"parity_exportAccount","params":["0x0042e5d2a662eeaca8a7e828c174f98f35d8925b","123"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32023,"message":"Could not export account.","data":"InvalidPassword"},"id":1}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));

	// correct password
	let request = r#"{"jsonrpc":"2.0","method":"parity_exportAccount","params":["0x0042e5d2a662eeaca8a7e828c174f98f35d8925b","parity-export-test"],"id":1}"#;

	let response = r#"{"jsonrpc":"2.0","result":{"address":"0042e5d2a662eeaca8a7e828c174f98f35d8925b","crypto":{"cipher":"aes-128-ctr","cipherparams":{"iv":"a1c6ff99070f8032ca1c4e8add006373"},"ciphertext":"df27e3db64aa18d984b6439443f73660643c2d119a6f0fa2fa9a6456fc802d75","kdf":"pbkdf2","kdfparams":{"c":10240,"dklen":32,"prf":"hmac-sha256","salt":"ddc325335cda5567a1719313e73b4842511f3e4a837c9658eeb78e51ebe8c815"},"mac":"3dc888ae79cbb226ff9c455669f6cf2d79be72120f2298f6cb0d444fddc0aa3d"},"id":"6a186c80-7797-cff2-bc2e-7c1d6a6cc76e","meta":"{\"passwordHint\":\"parity-export-test\",\"timestamp\":1490017814987}","name":"parity-export-test","version":3},"id":1}"#;
	let result = tester.io.handle_request_sync(&request);

	println!("Result: {:?}", result);
	println!("Response: {:?}", response);
	assert_eq!(result, Some(response.into()));
}

#[test]
fn should_import_wallet() {
	let tester = setup();

	let id = "6a186c80-7797-cff2-bc2e-7c1d6a6cc76e";
	let request = r#"{"jsonrpc":"2.0","method":"parity_newAccountFromWallet","params":["{\"id\":\"<ID>\",\"version\":3,\"crypto\":{\"cipher\":\"aes-128-ctr\",\"cipherparams\":{\"iv\":\"478736fb55872c1baf01b27b1998c90b\"},\"ciphertext\":\"fe5a63cc0055d7b0b3b57886f930ad9b63f48950d1348145d95996c41e05f4e0\",\"kdf\":\"pbkdf2\",\"kdfparams\":{\"c\":10240,\"dklen\":32,\"prf\":\"hmac-sha256\",\"salt\":\"658436d6738a19731149a98744e5cf02c8d5aa1f8e80c1a43cc9351c70a984e4\"},\"mac\":\"c7384b26ecf25539d942030230062af9b69de5766cbcc4690bffce1536644631\"},\"address\":\"00bac56a8a27232baa044c03f43bf3648c961735\",\"name\":\"hello world\",\"meta\":\"{}\"}", "himom"],"id":1}"#;
	let request = request.replace("<ID>", id);
	let response = r#"{"jsonrpc":"2.0","result":"0x00bac56a8a27232baa044c03f43bf3648c961735","id":1}"#;

	let res = tester.io.handle_request_sync(&request).unwrap();

	assert_eq!(res, response);

	let account_meta = tester.accounts.account_meta("0x00bac56a8a27232baa044c03f43bf3648c961735".into()).unwrap();
	let account_uuid: String = account_meta.uuid.unwrap().into();

	// the RPC should import the account with a new id
	assert!(account_uuid != id);
}

#[test]
fn should_sign_message() {
	let tester = setup();
	let hash = tester.accounts
		.insert_account(
			"0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a".parse().unwrap(),
			"password1")
		.expect("account should be inserted ok");

	assert_eq!(hash, "c171033d5cbff7175f29dfd3a63dda3d6f8f385e".parse().unwrap());

	let request = r#"{"jsonrpc": "2.0", "method": "parity_signMessage", "params": ["0xc171033d5cbff7175f29dfd3a63dda3d6f8f385e", "password1", "0xbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a"], "id": 3}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x1d9e33a8cf8bfc089a172bca01da462f9e359c6cb1b0f29398bc884e4d18df4f78588aee4fb5cc067ca62d2abab995e0bba29527be6ac98105b0320020a2efaf00","id":3}"#;
	let res = tester.io.handle_request_sync(&request);
	assert_eq!(res, Some(response.into()));
}
