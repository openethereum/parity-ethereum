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

use std::collections::BTreeMap;
use std::path::{Path,PathBuf};
use std::fs::File;
use serde::Deserialize;
use walkdir::{WalkDir,DirEntry};
use tempfile::tempdir;
use rayon::prelude::*;
use ethcore::log::{info,warn};
use globset::Glob;

#[derive(Debug, PartialEq, Deserialize)]
pub struct TestRunner {
	pub chain: Vec<ChainTests>,
	pub state: Vec<StateTests>,
	pub difficulty: Vec<DifficultyTests>,
	pub executive: Vec<ExecutiveTests>,
	pub transaction: Vec<TransactionTests>,
	pub trie: Vec<TrieTests>,
}

pub struct TestResult {
	pub success: usize,
	pub failed : Vec<String>,
}

impl TestResult {
	pub fn zero() -> Self {
		TestResult {
			success: 0,
			failed: Vec::new(),
		}
	}
	pub fn success() -> Self {
		TestResult {
			success: 1,
			failed: Vec::new(),
		}
	}
	pub fn failed(name: &str) -> Self {
		TestResult {
			success: 0,
			failed: vec![name.to_string()],
		}
	}
}

impl std::ops::Add for TestResult {
    type Output = Self;

    fn add(self, other: Self) -> Self {
		let mut mself = self;
		mself.success += other.success;
		mself.failed.extend_from_slice(&other.failed);
		mself
    }
}

impl std::ops::AddAssign for TestResult {
    fn add_assign(&mut self, other: Self) {
		self.success += other.success;
		self.failed.extend_from_slice(&other.failed);
    }
}

impl TestRunner {
	pub fn load<R>(reader: R) -> Result<Self, serde_json::Error> where R: std::io::Read {
		serde_json::from_reader(reader)
	}

	pub fn run_without_par(&self) -> TestResult {
		let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
		pool.install(|| self.run())
	}

	pub fn run(&self) -> TestResult {
		let mut res = TestResult::zero();
		for t in &self.chain {
			res += Self::run_chain_tests(&t);
		}
		for t in &self.state {
			res += Self::run_state_tests(&t);
		}
		for t in &self.difficulty {
			res += Self::run_difficuly_tests(&t);
		}
		for t in &self.executive {
			res += Self::run_executive_tests(&t);
		}
		for t in &self.transaction {
			res += Self::run_transaction_tests(&t);
		}
		for t in &self.trie {
			res += Self::run_trie_tests(&t);
		}
		res
	}

	fn find_json_files_recursive(path: &str) -> Vec<PathBuf> {
		WalkDir::new(path)
			.into_iter()
			.filter_map(|e| e.ok())
			.filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
			.map(DirEntry::into_path)
			.collect::<Vec<PathBuf>>()
	}

	fn report_failed_tests(path: &Path, list: &Vec<String>) {
		warn!("FAILED TESTS FOR {:?}: {:?} ",path,list);
	}

	fn run1<T,F>(test: &T, base_path: &str, f: F) -> TestResult
	where T : Send+Sync,
		  F : Fn(&T,&Path,&[u8])->Vec<String> + Send + Sync
	{
		let result = Self::find_json_files_recursive(&base_path)
			.into_par_iter()
			.map(|path| {
				info!("{:?}",path);
				let json = std::fs::read(&path).unwrap();
				let faileds = f(test, &path, &json);
				if faileds.len() > 0 {
					TestResult::failed(&faileds.join(","))
				} else {
					TestResult::success()
				}
			})
			.reduce(
				TestResult::zero,
				|a,b| a+b
			);

		if result.success + result.failed.len() == 0 {
			panic!("There is no tests in the specified path {}",base_path);
		}
		result
	}

	pub fn in_set(path: &Path, exprs: &[String]) -> bool {
		for pathexp in exprs {
			let glob = Glob::new(&pathexp).expect(&format!("cannot parse expression {}",pathexp)).compile_matcher();
			if glob.is_match(path) {
				return true;
			}
		}
		false
	}

	pub fn run_chain_tests(test: &ChainTests) -> TestResult {
		Self::run1(test, &test.path, |test: &ChainTests,path: &Path,json: &[u8]| {
			for skip in &test.skip {
				if Self::in_set(&path,&skip.paths) {
					println!("   - {} ..SKIPPED", path.to_string_lossy());
					return Vec::new();
				}
			}
			crate::chain::json_chain_test(&test, &path, &json, &mut |_,_| {}, false)
		})
	}
	pub fn run_state_tests(test: &StateTests) -> TestResult {
		Self::run1(test, &test.path, |test : &StateTests,path: &Path,json: &[u8]| {
			for skip in &test.skip {
				if Self::in_set(&path,&skip.paths) {
					println!("   - {} ..SKIPPED", path.to_string_lossy());
					return Vec::new();
				}
			}
			crate::state::json_chain_test(&test, &path, &json, &mut |_,_| {}, false)
		})
	}
	pub fn run_difficuly_tests(test: &DifficultyTests) -> TestResult {
		let mut acc = TestResult::zero();
		for path in &test.path {
			acc += Self::run1(test, &path, |test : &DifficultyTests,path : &Path,json : &[u8]| {
				let spec = match &test.chainspec {
					TestChainSpec::Foundation => ethcore::spec::new_foundation(&tempdir().unwrap().path()),
					TestChainSpec::ByzantiumTest => ethcore::spec::new_byzantium_test(),
					TestChainSpec::FrontierTest => ethcore::spec::new_frontier_test(),
					TestChainSpec::HomesteadTest => ethcore::spec::new_homestead_test(),
				};
				crate::difficulty::json_difficulty_test(&path, &json, spec, &mut |_,_| {})
			})
		}
		acc
	}

	pub fn run_executive_tests(test: &ExecutiveTests) -> TestResult {
		Self::run1(test, &test.path, |_ : &ExecutiveTests,path: &Path,json:&[u8]| {
			crate::executive::do_json_test(&path, &json, &mut |_,_| {})
		})
	}
	pub fn run_transaction_tests(test: &TransactionTests) -> TestResult {
		Self::run1(test, &test.path, |test :&TransactionTests,path:&Path,json: &[u8]| {
			crate::transaction::do_json_test(&path, &json, &mut |_,_| {})
		})
	}
	pub fn run_trie_tests(test: &TrieTests) -> TestResult {
		let mut acc = TestResult::zero();
		for path in &test.path {
			acc += Self::run1(test, &path,|test : &TrieTests,path: &Path,json: &[u8]| {
				let spec = match &test.triespec {
					TestTrieSpec::Generic => { println!("*GENERIC* [{:?}]",test.triespec); ethcore::trie::TrieSpec::Generic },
					TestTrieSpec::Secure => { println!("*SECURE* [{:?}]",test.triespec); ethcore::trie::TrieSpec::Secure },
				};
				crate::trie::test_trie(&path, &json, spec, &mut |_,_| {})
			});
		}
		acc
	}
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum TestTag {
	Heavy
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum TestChainSpec {
	Foundation,
	ByzantiumTest,
	FrontierTest,
	HomesteadTest,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum TestTrieSpec {
	Generic,
	Secure,
}

// --------------------------------------------------------------------

#[derive(Debug, PartialEq, Deserialize)]
pub struct ChainTests {
	pub path: String,
	pub skip: Vec<ChainTestSkip>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ChainTestSkip {
	/// Issue reference.
	pub reference: String,
	/// Items failing for the test.
	pub names: Vec<String>,
	pub paths: Vec<String>,
}

// --------------------------------------------------------------------

#[derive(Debug, PartialEq, Deserialize)]
pub struct StateTests {
	pub path: String,
	pub skip: Vec<StateTestSkip>,
}


#[derive(Debug, PartialEq, Deserialize)]
pub struct StateTestSkip{
	/// Issue reference.
	pub reference: String,
	/// Items failing for the test.
	pub paths: Vec<String>,
	pub names: BTreeMap<String, StateSkipSubStates>
}

/// State subtest to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateSkipSubStates {
	/// State test number of this item. Or '*' for all state.
	pub subnumbers: Vec<String>,
	/// Chain for this items.
	pub chain: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct DifficultyTests {
	pub path: Vec<String>,
	pub chainspec : TestChainSpec,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ExecutiveTests {
	pub path: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct TransactionTests {
	pub path: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct TrieTests {
	pub path: Vec<String>,
	pub triespec: TestTrieSpec,
}
