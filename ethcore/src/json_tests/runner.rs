use ethjson::test::{
    ChainTests, DifficultyTests, EthereumTestSuite, ExecutiveTests, StateTests, TestChainSpec,
    TestTrieSpec, TransactionTests, TrieTests,
};
use globset::Glob;
use log::info;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use trie::TrieSpec;

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

    fn run1<T, F>(test: &T, base_path: &PathBuf, f: F) -> TestResult
    where
        T: Send + Sync,
        F: Fn(&T, &Path, &[u8]) -> Vec<String> + Send + Sync,
    {
        let result = super::find_json_files_recursive(&base_path)
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
            panic!("There is no tests in the specified path {:?}", base_path);
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
                            crate::ethereum::new_foundation(&tempdir().unwrap().path())
                        }
                        TestChainSpec::ByzantiumTest => crate::ethereum::new_byzantium_test(),
                        TestChainSpec::FrontierTest => crate::ethereum::new_frontier_test(),
                        TestChainSpec::HomesteadTest => crate::ethereum::new_homestead_test(),
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
                super::executive::json_executive_test(&path, &json, &mut |_, _| {})
            },
        )
    }

    fn run_transaction_tests(test: &TransactionTests) -> TestResult {
        Self::run1(
            test,
            &test.path,
            |_: &TransactionTests, path: &Path, json: &[u8]| {
                super::transaction::json_transaction_test(&path, &json, &mut |_, _| {})
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
                super::trie::json_trie_test(&path, &json, spec, &mut |_, _| {})
            });
        }
        acc
    }
}

#[test]
fn ethereum_json_tests() {
    let content = std::fs::read("res/ethereum/runner/full.json")
        .expect("cannot open ethereum tests spec file");
    let runner =
        TestRunner::load(content.as_slice()).expect("cannot load ethereum tests spec file");
    println!("----------------------------------------------------");
    let result = match std::env::var_os("TEST_DEBUG") {
        Some(_) => runner.run_without_par(),
        _ => runner.run(),
    };
    println!("----------------------------------------------------");
    flushln!(
        "SUCCESS: {} FAILED: {} {:?}",
        result.success,
        result.failed.len(),
        result.failed
    );
    assert!(result.failed.len() == 0);
}
