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

pub use bigint::prelude::U256;
pub use bigint::hash::H256;
pub use util::*;
use std::collections::HashSet;
use std::io::Read;
use std::fs::{File, read_dir};
use std::path::Path;
use std::ffi::OsString;

pub fn run_test_path(p: &Path, skip: &[&'static str], runner: fn (json_data: &[u8]) -> Vec<String>) {
	let path = Path::new(p);
	let s: HashSet<OsString> = skip.iter().map(|s| {
		let mut os: OsString = s.into();
		os.push(".json");
		os
	}).collect();
	if path.is_dir() {
		for p in read_dir(path).unwrap().filter_map(|e| {
			let e = e.unwrap();
			if s.contains(&e.file_name()) {
				None
			} else {
				Some(e.path())
			}}) {
			run_test_path(&p, skip, runner)
		}
	} else {
		let mut path = p.to_path_buf();
		path.set_extension("json");
		run_test_file(&path, runner)
	}
}

pub fn run_test_file(path: &Path, runner: fn (json_data: &[u8]) -> Vec<String>) {
	let mut data = Vec::new();
	let mut file = File::open(&path).expect("Error opening test file");
	file.read_to_end(&mut data).expect("Error reading test file");
	let results = runner(&data);
	assert!(results.is_empty());
}

macro_rules! test {
	($name: expr, $skip: expr) => {
		::json_tests::test_common::run_test_path(::std::path::Path::new(concat!("res/ethereum/tests/", $name)), &$skip, do_json_test);
	}
}

#[macro_export]
macro_rules! declare_test {
	(skip => $arr: expr, $id: ident, $name: expr) => {
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, $arr);
		}
	};
	(ignore => $id: ident, $name: expr) => {
		#[ignore]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	};
	(heavy => $id: ident, $name: expr) => {
		#[cfg(feature = "test-heavy")]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	};
	($id: ident, $name: expr) => {
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	}
}
