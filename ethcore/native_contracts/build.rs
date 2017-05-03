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

// TODO: `include!` these from files where they're pretty-printed?
const REGISTRY_ABI: &'static str = r#"[{"constant":true,"inputs":[{"name":"_data","type":"address"}],"name":"canReverse","outputs":[{"name":"","type":"bool"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_new","type":"address"}],"name":"setOwner","outputs":[],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"bytes32"},{"name":"_key","type":"string"},{"name":"_value","type":"bytes32"}],"name":"setData","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"string"}],"name":"confirmReverse","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"bytes32"}],"name":"reserve","outputs":[{"name":"success","type":"bool"}],"payable":true,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"bytes32"}],"name":"drop","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_name","type":"bytes32"},{"name":"_key","type":"string"}],"name":"getAddress","outputs":[{"name":"","type":"address"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_amount","type":"uint256"}],"name":"setFee","outputs":[{"name":"","type":"bool"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"bytes32"},{"name":"_to","type":"address"}],"name":"transfer","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"owner","outputs":[{"name":"","type":"address"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_name","type":"bytes32"},{"name":"_key","type":"string"}],"name":"getData","outputs":[{"name":"","type":"bytes32"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_name","type":"bytes32"}],"name":"reserved","outputs":[{"name":"reserved","type":"bool"}],"payable":false,"type":"function"},{"constant":false,"inputs":[],"name":"drain","outputs":[{"name":"","type":"bool"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"string"},{"name":"_who","type":"address"}],"name":"proposeReverse","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_name","type":"bytes32"}],"name":"hasReverse","outputs":[{"name":"","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_name","type":"bytes32"},{"name":"_key","type":"string"}],"name":"getUint","outputs":[{"name":"","type":"uint256"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"fee","outputs":[{"name":"","type":"uint256"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_name","type":"bytes32"}],"name":"getOwner","outputs":[{"name":"","type":"address"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_name","type":"bytes32"}],"name":"getReverse","outputs":[{"name":"","type":"address"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_data","type":"address"}],"name":"reverse","outputs":[{"name":"","type":"string"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"bytes32"},{"name":"_key","type":"string"},{"name":"_value","type":"uint256"}],"name":"setUint","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"string"},{"name":"_who","type":"address"}],"name":"confirmReverseAs","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"},{"constant":false,"inputs":[],"name":"removeReverse","outputs":[],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_name","type":"bytes32"},{"name":"_key","type":"string"},{"name":"_value","type":"address"}],"name":"setAddress","outputs":[{"name":"success","type":"bool"}],"payable":false,"type":"function"}]"#;

const SERVICE_TRANSACTION_ABI: &'static str = r#"[{"constant":false,"inputs":[{"name":"_new","type":"address"}],"name":"setOwner","outputs":[],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_who","type":"address"}],"name":"certify","outputs":[],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_who","type":"address"},{"name":"_field","type":"string"}],"name":"getAddress","outputs":[{"name":"","type":"address"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_who","type":"address"}],"name":"revoke","outputs":[],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"owner","outputs":[{"name":"","type":"address"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"delegate","outputs":[{"name":"","type":"address"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_who","type":"address"},{"name":"_field","type":"string"}],"name":"getUint","outputs":[{"name":"","type":"uint256"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"_new","type":"address"}],"name":"setDelegate","outputs":[],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_who","type":"address"}],"name":"certified","outputs":[{"name":"","type":"bool"}],"payable":false,"type":"function"},{"constant":true,"inputs":[{"name":"_who","type":"address"},{"name":"_field","type":"string"}],"name":"get","outputs":[{"name":"","type":"bytes32"}],"payable":false,"type":"function"}]"#;

const SECRETSTORE_ACL_STORAGE_ABI: &'static str = r#"[{"constant":true,"inputs":[{"name":"user","type":"address"},{"name":"document","type":"bytes32"}],"name":"checkPermissions","outputs":[{"name":"","type":"bool"}],"payable":false,"type":"function"}]"#;

// be very careful changing these: ensure `ethcore/engines` validator sets have corresponding
// changes.
const VALIDATOR_SET_ABI: &'static str = r#"[{"constant":true,"inputs":[],"name":"transitionNonce","outputs":[{"name":"nonce","type":"uint256"}],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"getValidators","outputs":[{"name":"validators","type":"address[]"}],"payable":false,"type":"function"},{"anonymous":false,"inputs":[{"indexed":true,"name":"_parent_hash","type":"bytes32"},{"indexed":true,"name":"_nonce","type":"uint256"},{"indexed":false,"name":"_new_set","type":"address[]"}],"name":"ValidatorsChanged","type":"event"}]"#;

const VALIDATOR_REPORT_ABI: &'static str = r#"[{"constant":false,"inputs":[{"name":"validator","type":"address"},{"name":"blockNumber","type":"uint256"},{"name":"proof","type":"bytes"}],"name":"reportMalicious","outputs":[],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"validator","type":"address"},{"name":"blockNumber","type":"uint256"}],"name":"reportBenign","outputs":[],"payable":false,"type":"function"}]"#;

const TEST_VALIDATOR_SET_ABI: &'static str = r#"[{"constant":true,"inputs":[],"name":"transitionNonce","outputs":[{"name":"n","type":"uint256"}],"payable":false,"type":"function"},{"constant":false,"inputs":[{"name":"newValidators","type":"address[]"}],"name":"setValidators","outputs":[],"payable":false,"type":"function"},{"constant":true,"inputs":[],"name":"getValidators","outputs":[{"name":"vals","type":"address[]"}],"payable":false,"type":"function"},{"inputs":[],"payable":false,"type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"name":"_parent_hash","type":"bytes32"},{"indexed":true,"name":"_nonce","type":"uint256"},{"indexed":false,"name":"_new_set","type":"address[]"}],"name":"ValidatorsChanged","type":"event"}]"#;

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
	build_file("Registry", REGISTRY_ABI, "registry.rs");
	build_file("ServiceTransactionChecker", SERVICE_TRANSACTION_ABI, "service_transaction.rs");
	build_file("SecretStoreAclStorage", SECRETSTORE_ACL_STORAGE_ABI, "secretstore_acl_storage.rs");
	build_file("ValidatorSet", VALIDATOR_SET_ABI, "validator_set.rs");
	build_file("ValidatorReport", VALIDATOR_REPORT_ABI, "validator_report.rs");

	build_test_contracts();
}
