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

extern crate common_types;
extern crate serde;
extern crate docopt;

use common_types::BlockNumber;

use docopt::Docopt;
use serde::Deserialize;

const USAGE: &'static str = "
Parity Ethereum interaction with the DB on-demand.
  Copyright 2015-2019 Parity Technologies (UK) Ltd.

Usage:
    ethdb trace [--chain CHAIN --from=<block> --to=<block> --diff]
    ethdb extract [--chain CHAIN --from=<block> --to=<block> --block --receipts]
    ethdb state [--chain CHAIN --from=<block> --to=<block> --json --std-json  --std-err-only --std-out-only --std-dump-json]
    ethdb [-h | --help]
    ethdb --version

Commands:
    trace              Build TraceDB on-demand and add to node.
    extract            Extract data and output in JSON.
    state              State dump.

Trace options:
    --diff             Re-run block and produce state difference.

Extract options:
    --block            Block data.
    --receipts         Receipts.

General options:
    --chain CHAIN      Build only from specific chain.
    --from BLOCK       Build only from a specific block.
    --to BLOCK         Build only to a specific block.
    --json             Display verbose results in JSON.
    --std-json         Display results in standardized JSON format.
    --std-err-only     With --std-json redirect to err output only.
    --std-out-only     With --std-json redirect to out output only.
    --std-dump-json    Display results in standardized JSON format
                       with additional state dump.
    -h, --help         Display this message and exit.
";

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
