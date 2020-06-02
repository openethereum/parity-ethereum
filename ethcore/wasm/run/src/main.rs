// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate ethereum_types;
extern crate ethjson;
extern crate wasm;
extern crate vm;
extern crate clap;
extern crate rustc_hex;
extern crate env_logger;

mod fixture;
mod runner;

use fixture::Fixture;
use clap::Clap;
use std::fs;

#[derive(Clap)]
#[clap(
    name = "pwasm-run-test",
)]
struct Options {
	#[clap(
		required = true,
		name = "target",
		min_values = 1,
		about = "JSON fixture",
	)]
	pub target: Vec<String>
}

fn main() {
	::env_logger::init();

	let mut exit_code = 0;

	for target in Options::parse().target {
		let mut f = fs::File::open(&target).expect("Failed to open file");
		let fixtures: Vec<Fixture> = serde_json::from_reader(&mut f).expect("Failed to deserialize json");

		for fixture in fixtures.into_iter() {
			let fails = runner::run_fixture(&fixture);
			for fail in fails.iter() {
				exit_code = 1;
				println!("Failed assert in test \"{}\" ('{}'): {}", fixture.caption.as_ref(), target, fail);
			}
		}
	}

	std::process::exit(exit_code);
}
