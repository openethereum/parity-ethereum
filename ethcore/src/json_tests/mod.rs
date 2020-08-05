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

//! Helpers and tests for operating on jsontests.

#[macro_use]
mod test_common;

mod chain;
mod executive;
mod skip;
mod state;
mod transaction;
mod trie;

#[cfg(test)]
mod difficulty;

pub use self::test_common::HookType;

use self::skip::SKIP_TEST_STATE;
pub use self::{
    chain::{run_test_file as run_chain_test_file, run_test_path as run_chain_test_path},
    executive::{
        run_test_file as run_executive_test_file, run_test_path as run_executive_test_path,
    },
    state::{run_test_file as run_state_test_file, run_test_path as run_state_test_path},
    transaction::{
        run_test_file as run_transaction_test_file, run_test_path as run_transaction_test_path,
    },
    trie::{
        run_generic_test_file as run_generic_trie_test_file,
        run_generic_test_path as run_generic_trie_test_path,
        run_secure_test_file as run_secure_trie_test_file,
        run_secure_test_path as run_secure_trie_test_path,
    },
};
