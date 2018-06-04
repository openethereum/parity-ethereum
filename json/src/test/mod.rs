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

//! Additional test structures deserialization.

use std::collections::BTreeMap;
use std::io::Read;
use serde_json;
use serde_json::Error;
use hash::H256;
use uint::Uint;

/// Blockchain test header deserializer.
#[derive(Debug, PartialEq, Deserialize)]
pub struct DifficultyTestCase {
	/// Parent timestamp.
	#[serde(rename="parentTimestamp")]
	pub parent_timestamp: Uint,
	/// Parent difficulty.
	#[serde(rename="parentDifficulty")]
	pub parent_difficulty: Uint,
	/// Parent uncle hash.
	#[serde(rename="parentUncles")]
	pub parent_uncles: H256,
	/// Current timestamp.
	#[serde(rename="currentTimestamp")]
	pub current_timestamp: Uint,
	/// Current difficulty.
	#[serde(rename="currentDifficulty")]
	pub current_difficulty: Uint,
	/// Current block number.
	#[serde(rename="currentBlockNumber")]
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
	pub fn load<R>(reader: R) -> Result<Self, Error> where R: Read {
		serde_json::from_reader(reader)
	}
}
