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

use ethjson::{hash::H256, uint::Uint};
use serde::Deserialize;

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

/// Type for running `Difficulty` tests
pub type DifficultyTest = super::tester::GenericTester<String, DifficultyTestCase>;
