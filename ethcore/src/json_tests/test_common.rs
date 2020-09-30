// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

pub use ethereum_types::{Address, H256, U256};
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

/// Indicate when to run the hook passed to test functions.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum HookType {
    /// Hook to code to run on test start.
    OnStart,
    /// Hook to code to run on test end.
    OnStop,
}

/// find all json files recursively from a path
pub fn find_json_files_recursive(path: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .map(DirEntry::into_path)
        .collect::<Vec<PathBuf>>()
}

/// check if the test is selected to execute via TEST_DEBUG environment variable
pub fn debug_include_test(name: &str) -> bool {
    match std::env::var_os("TEST_DEBUG") {
        Some(s) => s.to_string_lossy().split_terminator(",").any(|expr| {
            regex::Regex::new(expr)
                .expect("invalid regex expression in TEST_DEBUG")
                .is_match(name)
        }),
        _ => true,
    }
}
