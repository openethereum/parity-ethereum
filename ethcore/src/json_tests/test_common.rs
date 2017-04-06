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

pub use util::*;
use std::fs::File;

pub fn run_test_file(path: &str, runner: fn (json_data: &[u8]) -> Vec<String>) {
	let mut data = Vec::new();
	let mut file = File::open(&path).expect("Error opening test file");
	file.read_to_end(&mut data).expect("Error reading test file");
	let results = runner(&data);
	assert!(results.is_empty());
}

macro_rules! test {
	($name: expr) => {
		::json_tests::test_common::run_test_file(concat!("res/ethereum/tests/", $name, ".json"), do_json_test);
	}
}

#[macro_export]
macro_rules! declare_test {
	(ignore => $id: ident, $name: expr) => {
		#[ignore]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	};
	(heavy => $id: ident, $name: expr) => {
		#[cfg(feature = "test-heavy")]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	};
	($id: ident, $name: expr) => {
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name);
		}
	}
}
