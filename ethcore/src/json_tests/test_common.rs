// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

/// Run all tests under the given path (except for the test files named in the skip list) using the
/// provided runner function.
pub fn run_test_path<H: FnMut(&str, HookType)>(
	path: &Path,
	skip: &[&'static str],
	runner: fn(path: &Path, json_data: &[u8], start_stop_hook: &mut H) -> Vec<String>,
	start_stop_hook: &mut H
) {
	if !skip.is_empty() {
		// todo[dvdplm] it's really annoying to have to use flushln here. Should be `info!(target:
		// "json-tests", …)`. Issue https://github.com/paritytech/parity-ethereum/issues/11084
		flushln!("[run_test_path] Skipping tests in {}: {:?}", path.display(), skip);
	}
	let mut errors = Vec::new();
	run_test_path_inner(path, skip, runner, start_stop_hook, &mut errors);
	let empty: [String; 0] = [];
	assert_eq!(errors, empty, "\nThere were {} tests in '{}' that failed.", errors.len(), path.display());
}

fn run_test_path_inner<H: FnMut(&str, HookType)>(
	p: &Path,
	skip: &[&'static str],
	runner: fn(path: &Path, json_data: &[u8], start_stop_hook: &mut H) -> Vec<String>,
	start_stop_hook: &mut H,
	errors: &mut Vec<String>
) {
	let path = Path::new(p);
	let extension = path.extension().and_then(|s| s.to_str());
	let skip_list: HashSet<OsString> = skip.iter().map(|s| {
		let mut os: OsString = s.into();
		os.push(".json");
		os
	}).collect();

	if path.is_dir() {
		trace!(target: "json-tests", "running tests contained in '{}'", path.display());
		let test_files = read_dir(path)
			.expect("Directory exists on disk")
			.filter_map(|dir_entry| {
				let dir_entry = dir_entry.expect("Entry in directory listing exists");
				if skip_list.contains(&dir_entry.file_name()) {
					debug!(target: "json-tests", "'{:?}' is on the skip list.", dir_entry.file_name());
					None
				} else {
					Some(dir_entry.path())
				}
			});
		for test_file in test_files {
			run_test_path_inner(&test_file, skip, runner, start_stop_hook, errors);
		}
	} else if extension == Some("swp") || extension == None {
		trace!(target: "json-tests", "ignoring '{}', extension {:?} – Junk?", path.display(), extension);
		// Ignore junk
	} else {
		trace!(target: "json-tests", "running tests in '{}'", path.display());
		let mut path = p.to_path_buf();
		path.set_extension("json");
		run_test_file_append(&path, runner, start_stop_hook, errors)
	}
}

fn run_test_file_append<H: FnMut(&str, HookType)>(
	path: &Path,
	runner: fn(path: &Path, json_data: &[u8], start_stop_hook: &mut H) -> Vec<String>,
	start_stop_hook: &mut H,
	errors: &mut Vec<String>
) {
	let mut data = Vec::new();
	let mut file = match File::open(&path) {
		Ok(file) => file,
		Err(_) => panic!("Error opening test file at: {:?}", path),
	};
	file.read_to_end(&mut data).expect("Error reading test file");
	errors.append(&mut runner(&path, &data, start_stop_hook));
}

pub fn run_test_file<H: FnMut(&str, HookType)>(
	path: &Path,
	runner: fn(path: &Path, json_data: &[u8], start_stop_hook: &mut H) -> Vec<String>,
	start_stop_hook: &mut H
) {
	let mut data = Vec::new();
	let mut file = match File::open(&path) {
		Ok(file) => file,
		Err(_) => panic!("Error opening test file at: {:?}", path),
	};
	file.read_to_end(&mut data).expect("Error reading test file");
	let results = runner(&path, &data, start_stop_hook);
	let empty: [String; 0] = [];
	assert_eq!(results, empty);
}

#[cfg(test)]
macro_rules! test {
	($name: expr, $skip: expr) => {
		::json_tests::test_common::run_test_path(
			::std::path::Path::new(concat!("res/ethereum/tests/", $name)),
			&$skip,
			do_json_test,
			&mut |_, _| ()
		);
	}
}

/// Declares a test:
///
/// declare_test!(test_name, "path/to/folder/with/tests");
///
/// Declares a test but skip the named test files inside the folder (no extension):
///
/// declare_test!(skip => ["a-test-file", "other-test-file"], test_name, "path/to/folder/with/tests");
///
/// NOTE: a skipped test is considered a passing test as far as `cargo test` is concerned. Normally
/// one test corresponds to a folder full of test files, each of which may contain many tests.
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
