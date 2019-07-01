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

//! Parity Ethereum command-line tool for database interaction.

#![warn(missing_docs)]

extern crate serde;
extern crate docopt;

use common_types::BlockNumber;

use docopt::Docopt;
use serde::Deserialize;

const USAGE: &'static str = include_str!("../res/USAGE");

#[derive(Debug, Deserialize)]
struct Args {
	cmd_trace: bool,
	cmd_extract: bool,
	cmd_state: bool,
	flag_chain: String,
	flag_from: BlockNumber,
	flag_to: BlockNumber,
	flag_diff: bool,
	flag_block: bool,
	flag_receipts: bool,
	flag_json: bool,
	flag_std_json: bool,
	flag_std_err_only: bool,
	flag_std_out_only: bool,
	flag_std_dump_json: bool,
}

fn main() {
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.deserialize())
		.unwrap_or_else(|e| e.exit());
	println!("{:?}", args);
	println!("Chain: {:?}", args.flag_chain);
}
