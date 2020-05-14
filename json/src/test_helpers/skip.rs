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
use serde::Deserialize;

/// Test to skip (only if issue ongoing)
#[derive(Debug, PartialEq, Deserialize)]
pub struct SkipTests {
	/// Block tests
	pub block: Vec<SkipBlockchainTest>,
	/// State tests
	pub state: Vec<SkipStateTest>,
	/// Legacy block tests
	pub legacy_block: Vec<SkipBlockchainTest>,
	/// Legacy state tests
	pub legacy_state: Vec<SkipStateTest>,

}

/// Block test to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct SkipBlockchainTest {
	/// Issue reference.
	pub reference: String,
	/// Test failing name.
	pub failing: String,
	/// Items failing for the test.
	pub subtests: Vec<String>,
}

/// State test to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct SkipStateTest {
	/// Issue reference.
	pub reference: String,
	/// Test failing name.
	pub failing: String,
	/// Items failing for the test.
	pub subtests: BTreeMap<String, StateSkipSubStates>
}

/// State subtest to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateSkipSubStates {
	/// State test number of this item. Or '*' for all state.
	pub subnumbers: Vec<String>,
	/// Chain for this items.
	pub chain: String,
}

impl SkipTests {
	/// Empty skip states.
	pub fn empty() -> Self {
		SkipTests {
			block: Vec::new(),
			state: Vec::new(),
			legacy_block: Vec::new(),
			legacy_state: Vec::new(),
		}
	}

	/// Loads test from json.
	pub fn load<R>(reader: R) -> Result<Self, serde_json::Error> where R: std::io::Read {
		serde_json::from_reader(reader)
	}
}
