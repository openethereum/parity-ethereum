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

//! Parity EVM interpreter binary.

#![warn(missing_docs)]
#![allow(dead_code)]
extern crate ethcore;
extern crate rustc_serialize;
extern crate docopt;
#[macro_use]
extern crate ethcore_util as util;

mod ext;

use std::sync::Arc;
use std::time::{Instant, Duration};
use std::fmt;
use std::str::FromStr;
use docopt::Docopt;
use util::{U256, FromHex, Uint, Bytes};
use ethcore::evm::{self, Factory, VMType, Finalize};
use ethcore::action_params::ActionParams;

const USAGE: &'static str = r#"
EVM implementation for Parity.
  Copyright 2016 Ethcore (UK) Limited

Usage:
    evmbin stats [options]
    evmbin [-h | --help]

Transaction options:
    --code CODE        Contract code as hex (without 0x)
    --input DATA       Input data as hex (without 0x)
    --gas GAS          Supplied gas as hex (without 0x)

General options:
    -h, --help         Display this message and exit.
"#;


fn main() {
	let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

	let mut params = ActionParams::default();
	params.gas = args.gas();
	params.code = Some(Arc::new(args.code()));
	params.data = args.data();

	let result = run_vm(params);
	match result {
		Ok(success) => println!("{}", success),
		Err(failure) => println!("{}", failure),
	}
}

/// Execute VM with given `ActionParams`
pub fn run_vm(params: ActionParams) -> Result<Success, Failure> {
	let initial_gas = params.gas;
	let factory = Factory::new(VMType::Interpreter, 1024);
	let mut vm = factory.create(params.gas);
	let mut ext = ext::FakeExt::default();

	let start = Instant::now();
	let gas_left = vm.exec(params, &mut ext).finalize(ext);
	let duration = start.elapsed();

	match gas_left {
		Ok(gas_left) => Ok(Success {
			gas_used: initial_gas - gas_left,
			// TODO [ToDr] get output from ext
			output: Vec::new(),
			time: duration,
		}),
		Err(e) => Err(Failure {
			error: e,
			time: duration,
		}),
	}
}

/// Execution finished correctly
pub struct Success {
	/// Used gas
	gas_used: U256,
	/// Output as bytes
	output: Vec<u8>,
	/// Time Taken
	time: Duration,
}
impl fmt::Display for Success {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		try!(writeln!(f, "Gas used: {:?}", self.gas_used));
		try!(writeln!(f, "Output: {:?}", self.output));
		try!(writeln!(f, "Time: {}.{:.9}s", self.time.as_secs(), self.time.subsec_nanos()));
		Ok(())
	}
}

/// Execution failed
pub struct Failure {
	/// Internal error
	error: evm::Error,
	/// Duration
	time: Duration,
}
impl fmt::Display for Failure {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		try!(writeln!(f, "Error: {:?}", self.error));
		try!(writeln!(f, "Time: {}.{:.9}s", self.time.as_secs(), self.time.subsec_nanos()));
		Ok(())
	}
}

#[derive(Debug, RustcDecodable)]
struct Args {
	cmd_stats: bool,
	flag_code: Option<String>,
	flag_gas: Option<String>,
	flag_input: Option<String>,
}

impl Args {
	pub fn gas(&self) -> U256 {
		self.flag_gas
			.clone()
			.and_then(|g| U256::from_str(&g).ok())
			.unwrap_or_else(|| !U256::zero())
	}

	pub fn code(&self) -> Bytes {
		self.flag_code
			.clone()
			.and_then(|c| c.from_hex().ok())
			.unwrap_or_else(|| die("Code is required."))
	}

	pub fn data(&self) -> Option<Bytes> {
		self.flag_input
			.clone()
			.and_then(|d| d.from_hex().ok())
	}
}

fn die(msg: &'static str) -> ! {
	println!("{}", msg);
	::std::process::exit(-1)
}
