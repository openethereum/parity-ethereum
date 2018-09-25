// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::collections::HashSet;
use std::io::Read;
use std::fs::{File, read_dir};
use std::path::Path;
use std::ffi::OsString;
pub use ethereum_types::{H256, U256, Address};

/// Indicate when to run the hook passed to test functions.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum HookType {
	/// Hook to code to run on test start.
	OnStart,
	/// Hook to code to run on test end.
	OnStop
}

pub fn run_test_path<H: FnMut(&str, HookType)>(
	p: &Path, skip: &[&'static str],
	runner: fn(json_data: &[u8], start_stop_hook: &mut H) -> Vec<String>,
	start_stop_hook: &mut H
) {
	let mut errors = Vec::new();
	run_test_path_inner(p, skip, runner, start_stop_hook, &mut errors);
	let empty: [String; 0] = [];
	assert_eq!(errors, empty);
}

fn run_test_path_inner<H: FnMut(&str, HookType)>(
	p: &Path, skip: &[&'static str],
	runner: fn(json_data: &[u8], start_stop_hook: &mut H) -> Vec<String>,
	start_stop_hook: &mut H,
	errors: &mut Vec<String>
) {
	let path = Path::new(p);
	let s: HashSet<OsString> = skip.iter().map(|s| {
		let mut os: OsString = s.into();
		os.push(".json");
		os
	}).collect();
	let extension = path.extension().and_then(|s| s.to_str());
	if path.is_dir() {
		for p in read_dir(path).unwrap().filter_map(|e| {
			let e = e.unwrap();
			if s.contains(&e.file_name()) {
				None
			} else {
				Some(e.path())
			}}) {
			run_test_path_inner(&p, skip, runner, start_stop_hook, errors);
		}
	} else if extension == Some("swp") || extension == None {
		// Ignore junk
	} else {
		let mut path = p.to_path_buf();
		path.set_extension("json");
		run_test_file_append(&path, runner, start_stop_hook, errors)
	}
}

fn run_test_file_append<H: FnMut(&str, HookType)>(
	path: &Path,
	runner: fn(json_data: &[u8], start_stop_hook: &mut H) -> Vec<String>,
	start_stop_hook: &mut H,
	errors: &mut Vec<String>
) {
	let mut data = Vec::new();
	let mut file = match File::open(&path) {
		Ok(file) => file,
		Err(_) => panic!("Error opening test file at: {:?}", path),
	};
	file.read_to_end(&mut data).expect("Error reading test file");
	errors.append(&mut runner(&data, start_stop_hook));
}

pub fn run_test_file<H: FnMut(&str, HookType)>(
	path: &Path,
	runner: fn(json_data: &[u8], start_stop_hook: &mut H) -> Vec<String>,
	start_stop_hook: &mut H
) {
	let mut data = Vec::new();
	let mut file = match File::open(&path) {
		Ok(file) => file,
		Err(_) => panic!("Error opening test file at: {:?}", path),
	};
	file.read_to_end(&mut data).expect("Error reading test file");
	let results = runner(&data, start_stop_hook);
	let empty: [String; 0] = [];
	assert_eq!(results, empty);
}

#[cfg(test)]
macro_rules! test {
	($name: expr, $skip: expr) => {
		::json_tests::test_common::run_test_path(::std::path::Path::new(concat!("res/ethereum/tests/", $name)), &$skip, do_json_test, &mut |_, _| ());
	}
}

#[macro_export]
macro_rules! declare_test {
	(skip => $arr: expr, $id: ident, $name: expr) => {
		#[cfg(test)]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, $arr);
		}
	};
	(ignore => $id: ident, $name: expr) => {
		#[cfg(test)]
		#[ignore]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	};
	(heavy => $id: ident, $name: expr) => {
		#[cfg(test)]
		#[cfg(feature = "test-heavy")]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	};
	($id: ident, $name: expr) => {
		#[cfg(test)]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	}
}
