// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Parity EVM Interpreter Binary.
//!
//! ## Overview
//!
//! The Parity EVM interpreter binary is a tool in the Parity
//! Ethereum toolchain. It is an EVM implementation for Parity Ethereum that
//! is used to run a standalone version of the EVM interpreter.
//!
//! ## Usage
//!
//! The evmbin tool is not distributed with regular Parity Ethereum releases
//! so you need to build it from source and run it like so:
//!
//! ```bash
//! cargo build -p evmbin --release
//! ./target/release/parity-evm --help
//! ```

#![warn(missing_docs)]

extern crate common_types as types;
extern crate ethcore;
extern crate ethjson;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate docopt;
extern crate parity_bytes as bytes;
extern crate ethereum_types;
extern crate vm;
extern crate evm;
extern crate panic_hook;
extern crate env_logger;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
extern crate tempdir;

use std::sync::Arc;
use std::{fmt, fs};
use std::path::PathBuf;
use docopt::Docopt;
use rustc_hex::FromHex;
use ethereum_types::{U256, Address};
use bytes::Bytes;
use ethcore::{spec, json_tests, TrieSpec};
use vm::{ActionParams, CallType};

pub mod info;
pub mod display;

use info::Informant;

const USAGE: &'static str = r#"
EVM implementation for Parity.
  Copyright 2015-2019 Parity Technologies (UK) Ltd.

Usage:
    parity-evm state-test <file> [--chain CHAIN --only NAME --json --std-json --std-dump-json --std-out-only --std-err-only]
    parity-evm stats [options]
    parity-evm stats-jsontests-vm <file>
    parity-evm [options]
    parity-evm [-h | --help]

Commands:
    state-test         Run a state test on a provided state test JSON file.
    stats              Execute EVM runtime code and return the statistics.
    stats-jsontests-vm Execute standard json-tests on a provided state test JSON
                       file path, format VMTests, and return timing statistics
                       in tsv format.

Transaction options:
    --code CODE        Contract code as hex (without 0x).
    --to ADDRESS       Recipient address (without 0x).
    --from ADDRESS     Sender address (without 0x).
    --input DATA       Input data as hex (without 0x).
    --gas GAS          Supplied gas as hex (without 0x).
    --gas-price WEI   Supplied gas price as hex (without 0x).

State test options:
    --chain CHAIN      Run only from specific chain name (i.e. one of EIP150, EIP158,
                       Frontier, Homestead, Byzantium, Constantinople,
                       ConstantinopleFix, EIP158ToByzantiumAt5, FrontierToHomesteadAt5,
                       HomesteadToDaoAt5, HomesteadToEIP150At5).
    --only NAME        Runs only a single test matching the name.

General options:
    --chain PATH       Path to chain spec file.
    --json             Display verbose results in JSON.
    --std-json         Display results in standardized JSON format.
    --std-dump-json    Display results in standardized JSON format
                       with additional state dump.
    --std-err-only     With --std-json redirect to err output only.
    --std-out-only     With --std-json redirect to out output only.
    -h, --help         Display this message and exit.
"#;

fn main() {
	panic_hook::set_abort();
	env_logger::init();

	let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());

	if args.cmd_state_test {
		run_state_test(args)
	} else if args.cmd_stats_jsontests_vm {
		run_stats_jsontests_vm(args)
	} else if args.flag_json {
		run_call(args, display::json::Informant::default())
	} else if args.flag_std_dump_json || args.flag_std_json {
		if args.flag_std_err_only {
			run_call(args, display::std_json::Informant::err_only())
		} else if args.flag_std_out_only {
			run_call(args, display::std_json::Informant::out_only())
		} else {
			run_call(args, display::std_json::Informant::default())
		};
	} else {
		run_call(args, display::simple::Informant::default())
	}
}

fn run_state_test(args: Args) {
	use ethjson::state::test::Test;

	// Parse the specified state test JSON file provided to the command `state-test <file>`.
	let file = args.arg_file.expect("PATH to a state test JSON file is required");
	let mut file = match fs::File::open(&file) {
		Err(err) => die(format!("Unable to open path: {:?}: {}", file, err)),
		Ok(file) => file,
	};
	let state_test = match Test::load(&mut file) {
		Err(err) => die(format!("Unable to load the test file: {}", err)),
		Ok(test) => test,
	};
	// Parse the name CLI option `--only NAME`.
	let only_test = args.flag_only.map(|s| s.to_lowercase());
	// Parse the chain `--chain CHAIN`
	let only_chain = args.flag_chain.map(|s| s.to_lowercase());

	// Iterate over 1st level (outer) key-value pair of the state test JSON file.
	// Skip to next iteration if CLI option `--only NAME` was parsed into `only_test` and does not match
	// the current key `name` (i.e. add11, create2callPrecompiles).
	for (name, test) in state_test {
		if let Some(false) = only_test.as_ref().map(|only_test| {
			&name.to_lowercase() == only_test
		}) {
			continue;
		}

		// Assign from 2nd level key-value pairs of the state test JSON file (i.e. env, post, pre, transaction).
		let multitransaction = test.transaction;
		let env_info = test.env.into();
		let pre = test.pre_state.into();

		// Iterate over remaining "post" key of the 2nd level key-value pairs in the state test JSON file.
		// Skip to next iteration if CLI option `--chain CHAIN` was parsed into `only_chain` and does not match
		// the current key `spec` (i.e. Constantinople, EIP150, EIP158).
		for (spec, states) in test.post_states {
			if let Some(false) = only_chain.as_ref().map(|only_chain| {
				&format!("{:?}", spec).to_lowercase() == only_chain
			}) {
				continue;
			}

			// Iterate over the 3rd level key-value pairs of the state test JSON file
			// (i.e. list of transactions and associated state roots hashes corresponding each chain).
			for (idx, state) in states.into_iter().enumerate() {
				let post_root = state.hash.into();
				let transaction = multitransaction.select(&state.indexes).into();

				// Determine the type of trie with state root to create in the database.
				// The database is a key-value datastore implemented as a database-backend
				// modified Merkle tree.
				// Use a secure trie database specification when CLI option `--std-dump-json`
				// is specified, otherwise use secure trie with fat trie database.
				let trie_spec = if args.flag_std_dump_json {
					TrieSpec::Fat
				} else {
					TrieSpec::Secure
				};

				// Execute the given transaction and verify resulting state root
				// for CLI option `--std-dump-json` or `--std-json`.
				if args.flag_std_dump_json || args.flag_std_json {
					if args.flag_std_err_only {
						// Use Standard JSON informant with err only
						info::run_transaction(&name, idx, &spec, &pre, post_root, &env_info, transaction, display::std_json::Informant::err_only(), trie_spec)
					} else if args.flag_std_out_only {
						// Use Standard JSON informant with out only
						info::run_transaction(&name, idx, &spec, &pre, post_root, &env_info, transaction, display::std_json::Informant::out_only(), trie_spec)
					} else {
						// Use Standard JSON informant default
						info::run_transaction(&name, idx, &spec, &pre, post_root, &env_info, transaction, display::std_json::Informant::default(), trie_spec)
					}
				} else {
					// Execute the given transaction and verify resulting state root
					// for CLI option `--json`.
					if args.flag_json {
						// Use JSON informant
						info::run_transaction(&name, idx, &spec, &pre, post_root, &env_info, transaction, display::json::Informant::default(), trie_spec)
					} else {
						// Use Simple informant
						info::run_transaction(&name, idx, &spec, &pre, post_root, &env_info, transaction, display::simple::Informant::default(), trie_spec)
					}
				}
			}
		}
	}
}

fn run_stats_jsontests_vm(args: Args) {
	use json_tests::HookType;
	use std::collections::HashMap;
	use std::time::{Instant, Duration};

	let file = args.arg_file.expect("PATH to a state test JSON file is required");

	let mut timings: HashMap<String, (Instant, Option<Duration>)> = HashMap::new();

	{
		let mut record_time = |name: &str, typ: HookType| {
			match typ {
				HookType::OnStart => {
					timings.insert(name.to_string(), (Instant::now(), None));
				},
				HookType::OnStop => {
					timings.entry(name.to_string()).and_modify(|v| {
						v.1 = Some(v.0.elapsed());
					});
				},
			}
		};
		if !file.is_file() {
			json_tests::run_executive_test_path(&file, &[], &mut record_time);
		} else {
			json_tests::run_executive_test_file(&file, &mut record_time);
		}
	}

	for (name, v) in timings {
		println!("{}\t{}", name, display::as_micros(&v.1.expect("All hooks are called with OnStop; qed")));
	}
}

// CLI command `stats`
fn run_call<T: Informant>(args: Args, informant: T) {
	let code = arg(args.code(), "--code");
	let to = arg(args.to(), "--to");
	let from = arg(args.from(), "--from");
	let data = arg(args.data(), "--input");
	let gas = arg(args.gas(), "--gas");
	let gas_price = arg(args.gas_price(), "--gas-price");
	let spec = arg(args.spec(), "--chain");

	if code.is_none() && to == Address::zero() {
		die("Either --code or --to is required.");
	}

	let mut params = ActionParams::default();
	params.call_type = if code.is_none() { CallType::Call } else { CallType::None };
	params.code = code.map(Arc::new);
	params.code_address = to;
	params.address = to;
	params.sender = from;
	params.origin = from;
	params.data = data;
	params.gas = gas;
	params.gas_price = gas_price;

	let mut sink = informant.clone_sink();
	let result = if args.flag_std_dump_json {
		info::run_action(&spec, params, informant, TrieSpec::Fat)
	} else {
		info::run_action(&spec, params, informant, TrieSpec::Secure)
	};
	T::finish(result, &mut sink);
}

#[derive(Debug, Deserialize)]
struct Args {
	cmd_stats: bool,
	cmd_state_test: bool,
	cmd_stats_jsontests_vm: bool,
	arg_file: Option<PathBuf>,
	flag_code: Option<String>,
	flag_to: Option<String>,
	flag_from: Option<String>,
	flag_input: Option<String>,
	flag_gas: Option<String>,
	flag_gas_price: Option<String>,
	flag_only: Option<String>,
	flag_chain: Option<String>,
	flag_json: bool,
	flag_std_json: bool,
	flag_std_dump_json: bool,
	flag_std_err_only: bool,
	flag_std_out_only: bool,
}

impl Args {
	// CLI option `--code CODE`
	/// Set the contract code in hex. Only send to either a contract code or a recipient address.
	pub fn code(&self) -> Result<Option<Bytes>, String> {
		match self.flag_code {
			Some(ref code) => code.from_hex().map(Some).map_err(to_string),
			None => Ok(None),
		}
	}

	// CLI option `--to ADDRESS`
	/// Set the recipient address in hex. Only send to either a contract code or a recipient address.
	pub fn to(&self) -> Result<Address, String> {
		match self.flag_to {
			Some(ref to) => to.parse().map_err(to_string),
			None => Ok(Address::zero()),
		}
	}

	// CLI option `--from ADDRESS`
	/// Set the sender address.
	pub fn from(&self) -> Result<Address, String> {
		match self.flag_from {
			Some(ref from) => from.parse().map_err(to_string),
			None => Ok(Address::zero()),
		}
	}

	// CLI option `--input DATA`
	/// Set the input data in hex.
	pub fn data(&self) -> Result<Option<Bytes>, String> {
		match self.flag_input {
			Some(ref input) => input.from_hex().map_err(to_string).map(Some),
			None => Ok(None),
		}
	}

	// CLI option `--gas GAS`
	/// Set the gas limit in units of gas. Defaults to max value to allow code to run for whatever time is required.
	pub fn gas(&self) -> Result<U256, String> {
		match self.flag_gas {
			Some(ref gas) => gas.parse().map_err(to_string),
			None => Ok(U256::from(u64::max_value())),
		}
	}

	// CLI option `--gas-price WEI`
	/// Set the gas price. Defaults to zero to allow the code to run even if an account with no balance
	/// is used, otherwise such accounts would not have sufficient funds to pay the transaction fee.
	/// Defaulting to zero also makes testing easier since it is not necessary to specify a special configuration file.
	pub fn gas_price(&self) -> Result<U256, String> {
		match self.flag_gas_price {
			Some(ref gas_price) => gas_price.parse().map_err(to_string),
			None => Ok(U256::zero()),
		}
	}

	// CLI option `--chain PATH`
	/// Set the path of the chain specification JSON file.
	pub fn spec(&self) -> Result<spec::Spec, String> {
		Ok(match self.flag_chain {
			Some(ref filename) => {
				let file = fs::File::open(filename).map_err(|e| format!("{}", e))?;
				spec::Spec::load(&::std::env::temp_dir(), file)?
			},
			None => {
				ethcore::ethereum::new_foundation(&::std::env::temp_dir())
			},
		})
	}
}

fn arg<T>(v: Result<T, String>, param: &str) -> T {
	v.unwrap_or_else(|e| die(format!("Invalid {}: {}", param, e)))
}

fn to_string<T: fmt::Display>(msg: T) -> String {
	format!("{}", msg)
}

fn die<T: fmt::Display>(msg: T) -> ! {
	println!("{}", msg);
	::std::process::exit(-1)
}

#[cfg(test)]
mod tests {
	use docopt::Docopt;
	use super::{Args, USAGE, Address};
	use ethjson::state::test::{State};

	fn run<T: AsRef<str>>(args: &[T]) -> Args {
		Docopt::new(USAGE).and_then(|d| d.argv(args.into_iter()).deserialize()).unwrap()
	}

	#[test]
	fn should_parse_all_the_options() {
		let args = run(&[
			"parity-evm",
			"--code", "05",
			"--to", "0000000000000000000000000000000000000004",
			"--from", "0000000000000000000000000000000000000003",
			"--input", "06",
			"--gas", "1",
			"--gas-price", "2",
			"--chain", "./testfile.json",
			"--json",
			"--std-json",
			"--std-dump-json",
			"--std-err-only",
			"--std-out-only",
		]);

		assert_eq!(args.code(), Ok(Some(vec![05])));
		assert_eq!(args.to(), Ok(Address::from_low_u64_be(4)));
		assert_eq!(args.from(), Ok(Address::from_low_u64_be(3)));
		assert_eq!(args.data(), Ok(Some(vec![06]))); // input data
		assert_eq!(args.gas(), Ok(1.into()));
		assert_eq!(args.gas_price(), Ok(2.into()));
		assert_eq!(args.flag_chain, Some("./testfile.json".to_owned()));
		assert_eq!(args.flag_json, true);
		assert_eq!(args.flag_std_json, true);
		assert_eq!(args.flag_std_dump_json, true);
		assert_eq!(args.flag_std_err_only, true);
		assert_eq!(args.flag_std_out_only, true);
	}

	#[test]
	fn should_parse_state_test_command() {
		let args = run(&[
			"parity-evm",
			"state-test",
			"./file.json",
			"--chain", "homestead",
			"--only=add11",
			"--json",
			"--std-json",
			"--std-dump-json",
			"--std-out-only",
			"--std-err-only",
		]);

		assert_eq!(args.cmd_state_test, true);
		assert!(args.arg_file.is_some());
		assert_eq!(args.flag_chain, Some("homestead".to_owned()));
		assert_eq!(args.flag_only, Some("add11".to_owned()));
		assert_eq!(args.flag_json, true);
		assert_eq!(args.flag_std_json, true);
		assert_eq!(args.flag_std_dump_json, true);
		assert_eq!(args.flag_std_out_only, true);
		assert_eq!(args.flag_std_err_only, true);
	}

	#[test]
	fn should_parse_state_test_command_from_state_test_json_file() {
		let s = r#"{
			"env": {
				"currentCoinbase": "2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
				"currentDifficulty": "0x0100",
				"currentGasLimit": "0x01c9c380",
				"currentNumber": "0x00",
				"currentTimestamp": "0x01",
				"previousHash": "5e20a0453cecd065ea59c37ac63e079ee08998b6045136a8ce6635c7912ec0b6"
			},
			"post": {
				"EIP150": [
					{
						"hash": "3e6dacc1575c6a8c76422255eca03529bbf4c0dda75dfc110b22d6dc4152396f",
						"indexes": { "data": 0, "gas": 0, "value": 0 }
					},
					{
						"hash": "99a450d8ce5b987a71346d8a0a1203711f770745c7ef326912e46761f14cd764",
						"indexes": { "data": 0, "gas": 0, "value": 1 }
					}
				],
				"EIP158": [
					{
						"hash": "3e6dacc1575c6a8c76422255eca03529bbf4c0dda75dfc110b22d6dc4152396f",
						"indexes": { "data": 0, "gas": 0, "value": 0 }
					},
					{
						"hash": "99a450d8ce5b987a71346d8a0a1203711f770745c7ef326912e46761f14cd764",
						"indexes": { "data": 0, "gas": 0, "value": 1  }
					}
				]
			},
			"pre": {
				"1000000000000000000000000000000000000000": {
					"balance": "0x0de0b6b3a7640000",
					"code": "0x6040600060406000600173100000000000000000000000000000000000000162055730f1600055",
					"nonce": "0x00",
					"storage": {
					}
				},
				"1000000000000000000000000000000000000001": {
					"balance": "0x0de0b6b3a7640000",
					"code": "0x604060006040600060027310000000000000000000000000000000000000026203d090f1600155",
					"nonce": "0x00",
					"storage": {
					}
				},
				"1000000000000000000000000000000000000002": {
					"balance": "0x00",
					"code": "0x600160025533600455346007553060e6553260e8553660ec553860ee553a60f055",
					"nonce": "0x00",
					"storage": {
					}
				},
				"a94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
					"balance": "0x0de0b6b3a7640000",
					"code": "0x",
					"nonce": "0x00",
					"storage": {
					}
				}
			},
			"transaction": {
				"data": [ "" ],
				"gasLimit": [ "285000", "100000", "6000" ],
				"gasPrice": "0x01",
				"nonce": "0x00",
				"secretKey": "45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8",
				"to": "095e7baea6a6c7c4c2dfeb977efac326af552d87",
				"value": [ "10", "0" ]
			}
		}"#;
		let _deserialized: State = serde_json::from_str(s).unwrap();
	}

	// TODO - add test that passes without failing with `State root mismatch`
	// using ./res/create2callPrecompile.json from https://github.com/ethereum/tests

  // TODO - add test that fails with `State root mismatch` using teststate.json

  // TODO - add test for the `parity-evm stats` command, and return error when
  // the `--only` option is used. repeat for `parity-evm stats-jsontests-vm`
  // and just `parity-evm` (since those options only supported
  // by `parity-evm state-test`)

  // TODO show out of gas error using only 1 gas, and when not out of gas by providing at least 21 gas.
  // ```
  // ./target/release/parity-evm stats --to "0000000000000000000000000000000000000004"
  // --from "0000000000000000000000000000000000000003" --code "05" --input "06" --gas "1"
  // --gas-price "2" --only "add11" --json
  // {"error":"EVM: Out of gas","gasUsed":"0x1","time":2422}
  //
  // ./target/release/parity-evm stats --to "0000000000000000000000000000000000000004"
  // --from "0000000000000000000000000000000000000003" --code "05" --input "06" --gas "21"
  // --gas-price "2" --only "add11" --json
  // {"gasUsed":"0x12","output":"0x06","time":2382}
  // ```
}
