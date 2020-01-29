// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

use std::sync::Arc;
use std::{fmt, fs};
use std::path::PathBuf;

use parity_bytes::Bytes;
use docopt::Docopt;
use rustc_hex::FromHex;
use ethereum_types::{U256, Address};
use ethcore::{json_tests, test_helpers::TrieSpec};
use spec;
use serde::Deserialize;
use vm::{ActionParams, ActionType};

mod info;
mod display;

use crate::info::{Informant, TxInput};

const USAGE: &'static str = r#"
EVM implementation for Parity.
  Copyright 2015-2020 Parity Technologies (UK) Ltd.

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
    --gas-price WEI    Supplied gas price as hex (without 0x).

State test options:
    --chain CHAIN      Run only from specific chain name (i.e. one of EIP150, EIP158,
                       Frontier, Homestead, Byzantium, Constantinople,
                       ConstantinopleFix, Istanbul, EIP158ToByzantiumAt5, FrontierToHomesteadAt5,
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
	use ethjson::test_helpers::state::Test;

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
	// the current key `state_test_name` (i.e. add11, create2callPrecompiles).
	for (state_test_name, test) in state_test {
		if let Some(false) = only_test.as_ref().map(|only_test| {
			&state_test_name.to_lowercase() == only_test
		}) {
			continue;
		}

		// Assign from 2nd level key-value pairs of the state test JSON file (i.e. env, post, pre, transaction).
		let multitransaction = test.transaction;
		let env_info = test.env.into();
		let pre = test.pre_state.into();

		// Iterate over remaining "post" key of the 2nd level key-value pairs in the state test JSON file.
		// Skip to next iteration if CLI option `--chain CHAIN` was parsed into `only_chain` and does not match
		// the current key `fork_spec_name` (i.e. Constantinople, EIP150, EIP158).
		for (fork_spec_name, states) in test.post_states {
			if let Some(false) = only_chain.as_ref().map(|only_chain| {
				&format!("{:?}", fork_spec_name).to_lowercase() == only_chain
			}) {
				continue;
			}

			// Iterate over the 3rd level key-value pairs of the state test JSON file
			// (i.e. list of transactions and associated state roots hashes corresponding each chain).
			for (tx_index, state) in states.into_iter().enumerate() {
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
						let tx_input = TxInput {
							state_test_name: &state_test_name,
							tx_index,
							fork_spec_name: &fork_spec_name,
							pre_state: &pre,
							post_root,
							env_info: &env_info,
							transaction,
							informant: display::std_json::Informant::err_only(),
							trie_spec,
						};
						// Use Standard JSON informant with err only
						info::run_transaction(tx_input);
					} else if args.flag_std_out_only {
						let tx_input = TxInput {
							state_test_name: &state_test_name,
							tx_index,
							fork_spec_name: &fork_spec_name,
							pre_state: &pre,
							post_root,
							env_info: &env_info,
							transaction,
							informant: display::std_json::Informant::out_only(),
							trie_spec,
						};
						// Use Standard JSON informant with out only
						info::run_transaction(tx_input);
					} else {
						let tx_input = TxInput {
							state_test_name: &state_test_name,
							tx_index,
							fork_spec_name: &fork_spec_name,
							pre_state: &pre,
							post_root,
							env_info: &env_info,
							transaction,
							informant: display::std_json::Informant::default(),
							trie_spec,
						};
						// Use Standard JSON informant default
						info::run_transaction(tx_input);
					}
				} else {
					// Execute the given transaction and verify resulting state root
					// for CLI option `--json`.
					if args.flag_json {
						let tx_input = TxInput {
							state_test_name: &state_test_name,
							tx_index,
							fork_spec_name: &fork_spec_name,
							pre_state: &pre,
							post_root,
							env_info: &env_info,
							transaction,
							informant: display::json::Informant::default(),
							trie_spec,
						};
						// Use JSON informant
						info::run_transaction(tx_input);
					} else {
						let tx_input = TxInput {
							state_test_name: &state_test_name,
							tx_index,
							fork_spec_name: &fork_spec_name,
							pre_state: &pre,
							post_root,
							env_info: &env_info,
							transaction,
							informant: display::simple::Informant::default(),
							trie_spec,
						};
						// Use Simple informant
						info::run_transaction(tx_input);
					}
				}
			}
		}
	}
}

fn run_stats_jsontests_vm(args: Args) {
	use crate::json_tests::HookType;
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
	params.action_type = if code.is_none() { ActionType::Call } else { ActionType::Create };
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
				let file = fs::File::open(filename).map_err(|e| e.to_string())?;
				spec::Spec::load(&::std::env::temp_dir(), file).map_err(|e| e.to_string())?
			},
			None => {
				spec::new_foundation(&::std::env::temp_dir())
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
	use common_types::transaction;
	use docopt::Docopt;
	use ethcore::test_helpers::TrieSpec;
	use ethjson::test_helpers::state::State;
	use serde::Deserialize;

	use super::{Args, USAGE, Address, run_call};
	use crate::{
		display::std_json::tests::informant,
		info::{self, TxInput}
	};

	#[derive(Debug, PartialEq, Deserialize)]
	pub struct SampleStateTests {
		pub add11: State,
		pub add12: State,
	}

	#[derive(Debug, PartialEq, Deserialize)]
	#[serde(rename_all = "camelCase")]
	pub struct ConstantinopleStateTests {
		pub create2call_precompiles: State,
	}

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
	#[should_panic]
	fn should_not_parse_only_flag_without_state_test() {
		let _ = run(&[
			"parity-evm",
			"./file.json",
			"--chain", "homestead",
			"--only=add11",
			"--json",
		]);
	}

	#[test]
	#[should_panic]
	fn should_not_parse_only_flag_with_stats() {
		let _ = run(&[
			"parity-evm",
			"stats",
			"./file.json",
			"--chain", "homestead",
			"--only=add11",
			"--json",
		]);
	}

	#[test]
	fn should_not_verify_state_root_using_sample_state_test_json_file() {
		let state_tests = include_str!("../res/teststate.json");
		// Parse the specified state test JSON file to simulate the CLI command `state-test <file>`.
		let deserialized_state_tests: SampleStateTests = serde_json::from_str(state_tests)
			.expect("Serialization cannot fail; qed");

		// Simulate the name CLI option `--only NAME`
		let state_test_name = "add11";
		let pre = deserialized_state_tests.add11.pre_state.into();
		let env_info = deserialized_state_tests.add11.env.into();
		let multitransaction = deserialized_state_tests.add11.transaction;

		for (fork_spec_name, tx_states) in deserialized_state_tests.add11.post_states.iter() {
			for (tx_index, tx_state) in tx_states.into_iter().enumerate() {
				let (informant, _, res) = informant();
				let trie_spec = TrieSpec::Secure;
				let transaction: transaction::SignedTransaction = multitransaction.select(&tx_state.indexes).into();
				let tx_input = TxInput {
					state_test_name: &state_test_name,
					tx_index,
					fork_spec_name: &fork_spec_name,
					pre_state: &pre,
					post_root: tx_states[tx_index].hash.0,
					env_info: &env_info,
					transaction,
					informant,
					trie_spec,
				};
				assert!(!info::run_transaction(tx_input));
				assert!(
					&String::from_utf8_lossy(&**res.0.lock().unwrap()).contains("State root mismatch")
				);
			}
		}
	}

	#[test]
	fn should_verify_state_root_using_constantinople_state_test_json_file() {
		let state_tests = include_str!("../res/create2callPrecompiles.json");
		// Parse the specified state test JSON file to simulate the CLI command `state-test <file>`.
		let deserialized_state_tests: ConstantinopleStateTests = serde_json::from_str(state_tests)
			.expect("Serialization cannot fail; qed");

		// Simulate the name CLI option `--only NAME`
		let state_test_name = "create2callPrecompiles";
		let pre = deserialized_state_tests.create2call_precompiles.pre_state.into();
		let env_info = deserialized_state_tests.create2call_precompiles.env.into();
		let multitransaction = deserialized_state_tests.create2call_precompiles.transaction;
		for (fork_spec_name, tx_states) in deserialized_state_tests.create2call_precompiles.post_states.iter() {
			for (tx_index, tx_state) in tx_states.into_iter().enumerate() {
				let (informant, _, _) = informant();
				let trie_spec = TrieSpec::Secure; // TrieSpec::Fat for --std_dump_json
				let transaction: transaction::SignedTransaction = multitransaction.select(&tx_state.indexes).into();
				let tx_input = TxInput {
					state_test_name: &state_test_name,
					tx_index,
					fork_spec_name: &fork_spec_name,
					pre_state: &pre,
					post_root: tx_states[tx_index].hash.0,
					env_info: &env_info,
					transaction,
					informant,
					trie_spec,
				};
				assert!(info::run_transaction(tx_input));
			}
		}
	}

	#[test]
	fn should_error_out_of_gas() {
		let args = run(&[
			"parity-evm",
			"stats",
			"--to", "0000000000000000000000000000000000000004",
			"--from", "0000000000000000000000000000000000000003",
			"--code", "05",
			"--input", "06",
			"--gas", "1",
			"--gas-price", "2",
			"--only=add11",
			"--std-json",
			"--std-out-only",
		]);

		let (inf, _, res) = informant();
		run_call(args, inf);

		assert!(
			&String::from_utf8_lossy(&**res.0.lock().unwrap())
				.starts_with(r#"{"error":"EVM: Out of gas","gasUsed":"0x1","#),
		);
	}

	#[test]
	fn should_not_error_out_of_gas() {
		let args = run(&[
			"parity-evm",
			"stats",
			"--to", "0000000000000000000000000000000000000004",
			"--from", "0000000000000000000000000000000000000003",
			"--code", "05",
			"--input", "06",
			"--gas", "21",
			"--gas-price", "2",
			"--only=add11",
			"--std-json",
			"--std-out-only",
		]);

		let (inf, _, res) = informant();
		run_call(args, inf);

		assert!(
			&String::from_utf8_lossy(&**res.0.lock().unwrap())
				.starts_with(r#"{"output":"0x06","gasUsed":"0x12","#),
		);
	}
}
