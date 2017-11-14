// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

extern crate native_contract_generator;

use std::path::Path;
use std::fs::File;
use std::io::Write;

// TODO: just walk the "res" directory and generate whole crate automatically.
const KEY_SERVER_SET_ABI: &'static str = include_str!("res/key_server_set.json");
const REGISTRY_ABI: &'static str = include_str!("res/registrar.json");
const URLHINT_ABI: &'static str = include_str!("res/urlhint.json");
const SERVICE_TRANSACTION_ABI: &'static str = include_str!("res/service_transaction.json");
const SECRETSTORE_ACL_STORAGE_ABI: &'static str = include_str!("res/secretstore_acl_storage.json");
const SECRETSTORE_SERVICE_ABI: &'static str = include_str!("res/secretstore_service.json");
const VALIDATOR_SET_ABI: &'static str = include_str!("res/validator_set.json");
const VALIDATOR_REPORT_ABI: &'static str = include_str!("res/validator_report.json");
const PEER_SET_ABI: &'static str = include_str!("res/peer_set.json");
const TX_ACL_ABI: &'static str = include_str!("res/tx_acl.json");

const TEST_VALIDATOR_SET_ABI: &'static str = include_str!("res/test_validator_set.json");

fn build_file(name: &str, abi: &str, filename: &str) {
	let code = ::native_contract_generator::generate_module(name, abi).unwrap();

	let out_dir = ::std::env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join(filename);
	let mut f = File::create(&dest_path).unwrap();

	f.write_all(code.as_bytes()).unwrap();
}

fn build_test_contracts() {
	build_file("ValidatorSet", TEST_VALIDATOR_SET_ABI, "test_validator_set.rs");
}

fn main() {
	build_file("KeyServerSet", KEY_SERVER_SET_ABI, "key_server_set.rs");
	build_file("Registry", REGISTRY_ABI, "registry.rs");
	build_file("Urlhint", URLHINT_ABI, "urlhint.rs");
	build_file("ServiceTransactionChecker", SERVICE_TRANSACTION_ABI, "service_transaction.rs");
	build_file("SecretStoreAclStorage", SECRETSTORE_ACL_STORAGE_ABI, "secretstore_acl_storage.rs");
	build_file("SecretStoreService", SECRETSTORE_SERVICE_ABI, "secretstore_service.rs");
	build_file("ValidatorSet", VALIDATOR_SET_ABI, "validator_set.rs");
	build_file("ValidatorReport", VALIDATOR_REPORT_ABI, "validator_report.rs");
	build_file("PeerSet", PEER_SET_ABI, "peer_set.rs");
	build_file("TransactAcl", TX_ACL_ABI, "tx_acl.rs");

	build_test_contracts();
}
