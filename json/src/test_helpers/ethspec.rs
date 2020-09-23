use serde::Deserialize;
use std::collections::BTreeMap;

/// Describes a github.com/ethereum/tests suite
#[derive(Debug, PartialEq, Deserialize)]
pub struct EthereumTestSuite {
	/// Blockchain tests
	pub chain: Vec<ChainTests>,
	/// State tests
	pub state: Vec<StateTests>,
	/// Difficulty tests
	pub difficulty: Vec<DifficultyTests>,
	/// Executive tests
	pub executive: Vec<ExecutiveTests>,
	/// Transaction tests
	pub transaction: Vec<TransactionTests>,
	/// Trie tests
	pub trie: Vec<TrieTests>,
}

/// Chain spec used in tests
#[derive(Debug, PartialEq, Deserialize)]
pub enum TestChainSpec {
	/// Foundation
	Foundation,
	/// ByzantiumTest
	ByzantiumTest,
	/// FrontierTest
	FrontierTest,
	/// HomesteadTest
	HomesteadTest,
}

/// Kind of trie used in test
#[derive(Debug, PartialEq, Deserialize)]
pub enum TestTrieSpec {
	/// Generic
	Generic,
	/// Secure
	Secure,
}

/// A set of blockchain tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct ChainTests {
	/// Path of the json tests
	pub path: String,
	/// Tests to skip
	pub skip: Vec<ChainTestSkip>,
}

/// Tests to skip in chain tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct ChainTestSkip {
	/// Issue reference.
	pub reference: String,
	/// Test names to skip
	pub names: Vec<String>,
	///  Test paths to skip
	pub paths: Vec<String>,
}

/// A set of state tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateTests {
	/// Path of the json tests
	pub path: String,
	/// Tests to skip
	pub skip: Vec<StateTestSkip>,
}

/// State test to skip
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateTestSkip{
	/// Issue reference.
	pub reference: String,
	/// Paths to skip
	pub paths: Vec<String>,
	/// Test names to skip
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

/// A set of difficulty tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct DifficultyTests {
	/// Path of the json tests
	pub path: Vec<String>,
	/// Chain spec to use
	pub chainspec : TestChainSpec,
}

/// A set of executive tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct ExecutiveTests {
	/// Path of the json tests
	pub path: String,
}

/// A set of transaction tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct TransactionTests {
	/// Path of the json tests
	pub path: String,
}

/// A set of trie tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct TrieTests {
	/// Path of the json tests
	pub path: Vec<String>,
	/// Trie spec to use
	pub triespec: TestTrieSpec,
}
