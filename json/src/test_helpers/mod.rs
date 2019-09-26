// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Test structures for JSON deserialization.

/// Blockchain test helpers
pub mod blockchain;
/// Difficulty test helpers
pub mod difficulty;
/// Tests to skip helpers
pub mod skip;
/// State test helpers
pub mod state;
/// Test primitives
pub mod tester;
/// Transaction test helpers
pub mod transaction;
/// Trie test helpers
pub mod trie;
/// Vm test helpers
pub mod vm {
	/// Type for running `vm` tests
	pub type Test = super::tester::GenericTester<String, crate::vm::Vm>;
}
