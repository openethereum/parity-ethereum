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

//! Parity EVM interpreter binary.

#![warn(missing_docs)]
#![allow(dead_code)]
extern crate ethcore;
extern crate rustc_hex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate ethcore_util as util;
extern crate vm;
extern crate evm;
extern crate panic_hook;

use std::sync::Arc;
use std::{fmt, fs};
use docopt::Docopt;
use rustc_hex::FromHex;
use util::{U256, Bytes, Address};
use ethcore::spec;
use vm::{ActionParams, CallType};

mod info;
mod display;

use info::Informant;

const USAGE: &'static str = r#"
EVM implementation for Parity.
  Copyright 2016, 2017 Parity Technologies (UK) Ltd

Usage:
    parity-evm stats [options]
    parity-evm [options]
    parity-evm [-h | --help]

Transaction options:
    --code CODE        Contract code as hex (without 0x).
    --to ADDRESS       Recipient address (without 0x).
    --from ADDRESS     Sender address (without 0x).
    --input DATA       Input data as hex (without 0x).
    --gas GAS          Supplied gas as hex (without 0x).
    --gas-price WEI    Supplied gas price as hex (without 0x).

General options:
    --json             Display verbose results in JSON.
    --chain CHAIN      Chain spec file path.
    -h, --help         Display this message and exit.
"#;


fn main() {
	panic_hook::set();

	let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());

	if args.flag_json {
		run(args, display::json::Informant::default())
	} else {
		run(args, display::simple::Informant::default())
	}
}

fn run<T: Informant>(args: Args, mut informant: T) {
	let from = arg(args.from(), "--from");
	let to = arg(args.to(), "--to");
	let code = arg(args.code(), "--code");
	let spec = arg(args.spec(), "--chain");
	let gas = arg(args.gas(), "--gas");
	let gas_price = arg(args.gas(), "--gas-price");
	let data = arg(args.data(), "--input");

	if code.is_none() && to == Address::default() {
		die("Either --code or --to is required.");
	}

	let mut params = ActionParams::default();
	params.call_type = if code.is_none() { CallType::Call } else { CallType::None };
	params.code_address = to;
	params.address = to;
	params.sender = from;
	params.origin = from;
	params.gas = gas;
	params.gas_price = gas_price;
	params.code = code.map(Arc::new);
	params.data = data;

	informant.set_gas(gas);
	let result = info::run(&mut informant, spec, params);
	informant.finish(result);
}

#[derive(Debug, Deserialize)]
struct Args {
	cmd_stats: bool,
	flag_from: Option<String>,
	flag_to: Option<String>,
	flag_code: Option<String>,
	flag_gas: Option<String>,
	flag_gas_price: Option<String>,
	flag_input: Option<String>,
	flag_chain: Option<String>,
	flag_json: bool,
}

impl Args {
	pub fn gas(&self) -> Result<U256, String> {
		match self.flag_gas {
			Some(ref gas) => gas.parse().map_err(to_string),
			None => Ok(!U256::zero()),
		}
	}

	pub fn gas_price(&self) -> Result<U256, String> {
		match self.flag_gas_price {
			Some(ref gas_price) => gas_price.parse().map_err(to_string),
			None => Ok(U256::zero()),
		}
	}

	pub fn from(&self) -> Result<Address, String> {
		match self.flag_from {
			Some(ref from) => from.parse().map_err(to_string),
			None => Ok(Address::default()),
		}
	}

	pub fn to(&self) -> Result<Address, String> {
		match self.flag_to {
			Some(ref to) => to.parse().map_err(to_string),
			None => Ok(Address::default()),
		}
	}

	pub fn code(&self) -> Result<Option<Bytes>, String> {
		match self.flag_code {
			Some(ref code) => code.from_hex().map(Some).map_err(to_string),
			None => Ok(None),
		}
	}

	pub fn data(&self) -> Result<Option<Bytes>, String> {
		match self.flag_input {
			Some(ref input) => input.from_hex().map_err(to_string).map(Some),
			None => Ok(None),
		}
	}

	pub fn spec(&self) -> Result<spec::Spec, String> {
		Ok(match self.flag_chain {
			Some(ref filename) =>  {
				let file = fs::File::open(filename).map_err(|e| format!("{}", e))?;
				spec::Spec::load(::std::env::temp_dir(), file)?
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
	use super::{Args, USAGE};

	fn run<T: AsRef<str>>(args: &[T]) -> Args {
		Docopt::new(USAGE).and_then(|d| d.argv(args.into_iter()).deserialize()).unwrap()
	}

	#[test]
	fn should_parse_all_the_options() {
		let args = run(&[
			"parity-evm",
			"--json",
			"--gas", "1",
			"--gas-price", "2",
			"--from", "0000000000000000000000000000000000000003",
			"--to", "0000000000000000000000000000000000000004",
			"--code", "05",
			"--input", "06",
			"--chain", "./testfile",
		]);

		assert_eq!(args.flag_json, true);
		assert_eq!(args.gas(), Ok(1.into()));
		assert_eq!(args.gas_price(), Ok(2.into()));
		assert_eq!(args.from(), Ok(3.into()));
		assert_eq!(args.to(), Ok(4.into()));
		assert_eq!(args.code(), Ok(Some(vec![05])));
		assert_eq!(args.data(), Ok(Some(vec![06])));
		assert_eq!(args.flag_chain, Some("./testfile".to_owned()));
	}
}
