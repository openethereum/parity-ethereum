// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! State backends. These facilitate querying of state data by the `State` structure.

mod db;

use util::{Address, H256, Bytes};
use std::collections::HashMap;
use std::sync::Arc;

use error::Error;
use state::{Account, AccountEntry};

pub use self::db::Database;

/// The state backend trait.
/// This is intended to provide a generic abstraction over disk-backed state queries as
/// well as network-backed ones.
pub trait Backend: Clone + Send {
	/// Query an account's contract code.
	/// Returns `None` if it doesn't exist.
	fn code(&self, address: Address, code_hash: &H256) -> Option<Arc<Bytes>>;

	/// Query an account with the given state root and address.
	/// Returns the RLP of the account structure or `None` if it doesn't exist.
	fn account(&self, address: &Address) -> Option<Account>;

	/// Query an account's storage by key.
	/// Returns `None` if it doesn't exist.
	fn storage(&self, address: Address, storage_root: &H256, key: &H256) -> H256;

	// TODO: optimize API to support combining account and storage queries into single
	// GetProofs packet for LES.

	/// Commit all the accounts and their storage from the given cache, marking them clean
	/// as it goes.
	fn commit(&mut self, accounts: &mut HashMap<Address, AccountEntry>) -> Result<(), Error>;

	/// Get the state root.
	fn root(&self) -> &H256;
}