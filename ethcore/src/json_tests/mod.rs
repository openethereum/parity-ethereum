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

//! Helpers and tests for operating on `JSON` tests.

#[macro_use]
mod macros;

mod chain;
mod executive;
mod skip;
mod state;
mod test_common;
mod transaction;
mod trie;

#[cfg(test)]
mod difficulty;

pub use self::executive::run_test_path as run_executive_test_path;
pub use self::executive::run_test_file as run_executive_test_file;
pub use self::test_common::HookType;

use self::skip::SKIP_TESTS;
