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

//! Trie test deserialization.

use hash::H256;
use trie::Input;

/// Trie test deserialization.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Trie {
	/// Trie test input.
	#[serde(rename = "in")]
	pub input: Input,
	/// Trie root hash.
	pub root: H256,
}
