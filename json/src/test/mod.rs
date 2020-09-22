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

//! Additional test structures deserialization.

use hash::H256;
use serde_json::{self, Error};
use std::{collections::BTreeMap, io::Read, path::PathBuf};
use uint::Uint;

/// Blockchain test header deserializer.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyTestCase {
    /// Parent timestamp.
    pub parent_timestamp: Uint,
    /// Parent difficulty.
    pub parent_difficulty: Uint,
    /// Parent uncle hash.
    pub parent_uncles: H256,
    /// Current timestamp.
    pub current_timestamp: Uint,
    /// Current difficulty.
    pub current_difficulty: Uint,
    /// Current block number.
    pub current_block_number: Uint,
}

/// Blockchain test deserializer.
#[derive(Debug, PartialEq, Deserialize)]
pub struct DifficultyTest(BTreeMap<String, DifficultyTestCase>);

impl IntoIterator for DifficultyTest {
    type Item = <BTreeMap<String, DifficultyTestCase> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<String, DifficultyTestCase> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl DifficultyTest {
    /// Loads test from json.
    pub fn load<R>(reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        serde_json::from_reader(reader)
    }
}

/// Test to skip (only if issue ongoing)
#[derive(Debug, PartialEq, Deserialize)]
pub struct SkipStates {
    /// Block tests
    pub block: Vec<BlockSkipStates>,
    /// State tests
    pub state: Vec<StateSkipStates>,
}

/// Block test to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct BlockSkipStates {
    /// Issue reference.
    pub reference: String,
    /// Test failing name.
    pub failing: String,
    /// Items failing for the test.
    pub subtests: Vec<String>,
}

/// State test to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateSkipStates {
    /// Issue reference.
    pub reference: String,
    /// Test failing name.
    pub failing: String,
    /// Items failing for the test.
    pub subtests: BTreeMap<String, StateSkipSubStates>,
}

/// State subtest to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateSkipSubStates {
    /// State test number of this item. Or '*' for all state.
    pub subnumbers: Vec<String>,
    /// Chain for this items.
    pub chain: String,
}

impl SkipStates {
    /// Loads skip states from json.
    pub fn load<R>(reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        serde_json::from_reader(reader)
    }

    /// Empty skip states.
    pub fn empty() -> Self {
        SkipStates {
            block: Vec::new(),
            state: Vec::new(),
        }
    }
}

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
    pub path: PathBuf,
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
    pub path: PathBuf,
    /// Tests to skip
    pub skip: Vec<StateTestSkip>,
}

/// State test to skip
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateTestSkip {
    /// Issue reference.
    pub reference: String,
    /// Paths to skip
    pub paths: Vec<String>,
    /// Test names to skip
    pub names: BTreeMap<String, StateSkipSubStates1>,
}

/// State subtest to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateSkipSubStates1 {
    /// State test number of this item. Or '*' for all state.
    pub subnumbers: Vec<String>,
    /// Chain for this items.
    pub chain: String,
}

/// A set of difficulty tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct DifficultyTests {
    /// Path of the json tests
    pub path: Vec<PathBuf>,
    /// Chain spec to use
    pub chainspec: TestChainSpec,
}

/// A set of executive tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct ExecutiveTests {
    /// Path of the json tests
    pub path: PathBuf,
}

/// A set of transaction tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct TransactionTests {
    /// Path of the json tests
    pub path: PathBuf,
}

/// A set of trie tests
#[derive(Debug, PartialEq, Deserialize)]
pub struct TrieTests {
    /// Path of the json tests
    pub path: Vec<PathBuf>,
    /// Trie spec to use
    pub triespec: TestTrieSpec,
}
