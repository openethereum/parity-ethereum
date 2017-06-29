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
extern crate rustc_serialize;
extern crate docopt;
extern crate ethcore_util as util;

use std::sync::Arc;
use std::{fmt, fs};
use docopt::Docopt;
use util::{U256, FromHex, Bytes, Address};
use ethcore::spec;
use ethcore::action_params::ActionParams;

mod vm;
mod display;

use vm::Informant;

const USAGE: &'static str = r#"
EVM implementation for Parity.
  Copyright 2016, 2017 Parity Technologies (UK) Ltd

Usage:
    evmbin stats [options]
    evmbin [options]
    evmbin [-h | --help]

Transaction options:
    --code CODE        Contract code as hex (without 0x).
    --from ADDRESS     Sender address (without 0x).
    --input DATA       Input data as hex (without 0x).
    --gas GAS          Supplied gas as hex (without 0x).

General options:
    --json             Display verbose results in JSON.
    --chain CHAIN      Chain spec file path.
    -h, --help         Display this message and exit.
"#;


fn main() {
	let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

	if args.flag_json {
		run(args, display::json::Informant::default())
	} else {
		run(args, display::simple::Informant::default())
	}
}

fn run<T: Informant>(args: Args, mut informant: T) {
	let from = arg(args.from(), "--from");
	let code = arg(args.code(), "--code");
	let spec = arg(args.spec(), "--chain");
	let gas = arg(args.gas(), "--gas");
	let data = arg(args.data(), "--input");

	let mut params = ActionParams::default();
	params.sender = from;
	params.origin = from;
	params.gas = gas;
	params.code = Some(Arc::new(code));
	params.data = data;

	informant.set_gas(gas);
	let result = vm::run(&mut informant, spec, params);
	informant.finish(result);
}

#[derive(Debug, RustcDecodable)]
struct Args {
	cmd_stats: bool,
	flag_from: Option<String>,
	flag_code: Option<String>,
	flag_gas: Option<String>,
	flag_input: Option<String>,
	flag_spec: Option<String>,
	flag_json: bool,
}

impl Args {
	pub fn gas(&self) -> Result<U256, String> {
		match self.flag_gas {
			Some(ref gas) => gas.parse().map_err(to_string),
			None => Ok(!U256::zero()),
		}
	}

	pub fn from(&self) -> Result<Address, String> {
		match self.flag_from {
			Some(ref from) => from.parse().map_err(to_string),
			None => Ok(Address::default()),
		}
	}

	pub fn code(&self) -> Result<Bytes, String> {
		match self.flag_code {
			Some(ref code) => code.from_hex().map_err(to_string),
			None => Err("Code is required!".into()),
		}
	}

	pub fn data(&self) -> Result<Option<Bytes>, String> {
		match self.flag_input {
			Some(ref input) => input.from_hex().map_err(to_string).map(Some),
			None => Ok(None),
		}
	}

	pub fn spec(&self) -> Result<spec::Spec, String> {
		Ok(match self.flag_spec {
			Some(ref filename) =>  {
				let file = fs::File::open(filename).map_err(|e| format!("{}", e))?;
				spec::Spec::load(file)?
			},
			None => {
				ethcore::ethereum::new_foundation()
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
