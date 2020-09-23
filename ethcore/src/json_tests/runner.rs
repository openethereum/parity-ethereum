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
#![allow(dead_code)]

use ethjson::test_helpers::ethspec::{
    ChainTests, DifficultyTests, EthereumTestSuite, ExecutiveTests, StateTests, TestChainSpec,
    TestTrieSpec, TransactionTests, TrieTests,
};
use globset::Glob;
use log::info;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use trie::TrieSpec;
use walkdir::{DirEntry, WalkDir};

/// Result of tests execution
pub struct TestResult {
    /// Number of success execution
    pub success: usize,
    /// Number of success execution
    pub failed: Vec<String>,
}

impl TestResult {
    /// Creates a new TestResult without results
    pub fn zero() -> Self {
        TestResult {
            success: 0,
            failed: Vec::new(),
        }
    }
    /// Creates a new success TestResult 
    pub fn success() -> Self {
        TestResult {
            success: 1,
            failed: Vec::new(),
        }
    }
    /// Creates a new failed TestResult 
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

pub struct TestRunner(EthereumTestSuite);

impl TestRunner {
    /// Loads a new JSON Test suite
    pub fn load<R>(reader: R) -> Result<Self, serde_json::Error>
    where
        R: std::io::Read,
    {
        Ok(TestRunner(serde_json::from_reader(reader)?))
    }

    /// Run the tests with one thread
    pub fn run_without_par(&self) -> TestResult {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap();
        pool.install(|| self.run())
    }

    /// Run the tests
    pub fn run(&self) -> TestResult {
        let mut res = TestResult::zero();
        for t in &self.0.chain {
            res += Self::run_chain_tests(&t);
        }
        for t in &self.0.state {
            res += Self::run_state_tests(&t);
        }
        for t in &self.0.difficulty {
            res += Self::run_difficuly_tests(&t);
        }
        for t in &self.0.executive {
            res += Self::run_executive_tests(&t);
        }
        for t in &self.0.transaction {
            res += Self::run_transaction_tests(&t);
        }
        for t in &self.0.trie {
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

    fn run1<T, F>(test: &T, base_path: &str, f: F) -> TestResult
    where
        T: Send + Sync,
        F: Fn(&T, &Path, &[u8]) -> Vec<String> + Send + Sync,
    {
        let result = Self::find_json_files_recursive(&base_path)
            .into_par_iter()
            .map(|path| {
                info!("{:?}", path);
                let json = std::fs::read(&path).unwrap();
                let faileds = f(test, &path, &json);
                if faileds.len() > 0 {
                    TestResult::failed(&faileds.join(","))
                } else {
                    TestResult::success()
                }
            })
            .reduce(TestResult::zero, |a, b| a + b);

        if result.success + result.failed.len() == 0 {
            panic!("There is no tests in the specified path {}", base_path);
        }
        result
    }

    fn in_set(path: &Path, exprs: &[String]) -> bool {
        for pathexp in exprs {
            let glob = Glob::new(&pathexp)
                .expect(&format!("cannot parse expression {}", pathexp))
                .compile_matcher();
            if glob.is_match(path) {
                return true;
            }
        }
        false
    }

    fn run_chain_tests(test: &ChainTests) -> TestResult {
        Self::run1(
            test,
            &test.path,
            |test: &ChainTests, path: &Path, json: &[u8]| {
                for skip in &test.skip {
                    if Self::in_set(&path, &skip.paths) {
                        println!("   - {} ..SKIPPED", path.to_string_lossy());
                        return Vec::new();
                    }
                }
                super::chain::json_chain_test(&test, &path, &json, &mut |_, _| {})
            },
        )
    }

    fn run_state_tests(test: &StateTests) -> TestResult {
        Self::run1(
            test,
            &test.path,
            |test: &StateTests, path: &Path, json: &[u8]| {
                for skip in &test.skip {
                    if Self::in_set(&path, &skip.paths) {
                        println!("   - {} ..SKIPPED", path.to_string_lossy());
                        return Vec::new();
                    }
                }
                super::state::json_chain_test(&test, &path, &json, &mut |_, _| {})
            },
        )
    }

    fn run_difficuly_tests(test: &DifficultyTests) -> TestResult {
        let mut acc = TestResult::zero();
        for path in &test.path {
            acc += Self::run1(
                test,
                &path,
                |test: &DifficultyTests, path: &Path, json: &[u8]| {
                    let spec = match &test.chainspec {
                        TestChainSpec::Foundation => {
                            crate::spec::new_foundation(&tempdir().unwrap().path())
                        }
                        TestChainSpec::ByzantiumTest => crate::spec::new_byzantium_test(),
                        TestChainSpec::FrontierTest => crate::spec::new_frontier_test(),
                        TestChainSpec::HomesteadTest => crate::spec::new_homestead_test(),
                    };
                    super::difficulty::json_difficulty_test(&path, &json, spec, &mut |_, _| {})
                },
            )
        }
        acc
    }

    fn run_executive_tests(test: &ExecutiveTests) -> TestResult {
        Self::run1(
            test,
            &test.path,
            |_: &ExecutiveTests, path: &Path, json: &[u8]| {
                super::executive::do_json_test(&path, &json, &mut |_, _| {})
            },
        )
    }

    fn run_transaction_tests(test: &TransactionTests) -> TestResult {
        Self::run1(
            test,
            &test.path,
            |_: &TransactionTests, path: &Path, json: &[u8]| {
                super::transaction::do_json_test(&path, &json, &mut |_, _| {})
            },
        )
    }

    fn run_trie_tests(test: &TrieTests) -> TestResult {
        let mut acc = TestResult::zero();
        for path in &test.path {
            acc += Self::run1(test, &path, |test: &TrieTests, path: &Path, json: &[u8]| {
                let spec = match &test.triespec {
                    TestTrieSpec::Generic => TrieSpec::Generic,
                    TestTrieSpec::Secure => TrieSpec::Secure,
                };
                super::trie::test_trie(&path, &json, spec, &mut |_, _| {})
            });
        }
        acc
    }
}

#[cfg(test)]
mod test {
    use super::TestRunner;
    #[test]
    fn test_ethereum_json_tests() {
        let content = std::fs::read("res/ethereum/runner/full.json")
            .expect("cannot open ethreum tests spec file");
        let runner = TestRunner::load(content.as_slice()).expect("cannot load content");
        let result = runner.run();
        println!(
            "SUCCESS: {} FAILED: {} {:?}",
            result.success,
            result.failed.len(),
            result.failed
        );
        assert!(result.failed.len() == 0);
    }
}
