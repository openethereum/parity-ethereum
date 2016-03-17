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

extern crate docopt;
extern crate rustc_serialize;
extern crate serde_json;
extern crate ethjson;
extern crate ethcore;

use std::process;
use std::fs::File;
use std::path::Path;
use docopt::Docopt;
use ethcore::spec::Genesis;
use ethcore::pod_state::PodState;
use ethcore::ethereum;

const USAGE: &'static str = r#"
Parity rpctest client.
  By Wood/Paronyan/Kotewicz/Drwięga/Volf.
  Copyright 2015, 2016 Ethcore (UK) Limited

Usage:
  parity --json <test-file> --name <test-name>
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
	arg_test_file: String,
	arg_test_name: String,
}

struct Configuration {
	args: Args,
}

impl Configuration {
	fn parse() -> Self {
		Configuration {
			args: Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit())
		}
	}

	fn execute(&self) {
		println!("file path: {:?}", self.args.arg_test_file);
		println!("test name: {:?}", self.args.arg_test_name);

		let path = Path::new(&self.args.arg_test_file);
		let file = File::open(path).unwrap_or_else(|_| {
			println!("Cannot open file.");
			process::exit(1);
		});

		let	tests: ethjson::blockchain::Test = serde_json::from_reader(file).unwrap_or_else(|_| {
			println!("Invalid json file.");
			process::exit(2);
		});

		let blockchain = tests.get(&self.args.arg_test_name).unwrap_or_else(|| {
			println!("Invalid test name.");
			process::exit(3);
		});

		let genesis = Genesis::from(blockchain.genesis());
		let state = PodState::from(blockchain.pre_state.clone());
		let mut spec = ethereum::new_frontier_test();
		spec.set_genesis_state(state);
		spec.overwrite_genesis_params(genesis);
		assert!(spec.is_state_root_valid());

		//let temp = RandomTempPath::new();
		//spec.

	}
}

fn main() {
	Configuration::parse().execute();
}
