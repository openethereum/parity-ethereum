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
use util::{Address, H256, Bytes};


/// The state backend trait.
/// This is intended to provide a generic abstraction over disk-backed state queries as
/// well as network-backed ones.
pub trait Backend {
	/// Query an account's contract code.
	/// Returns `None` if it doesn't exist.
	fn code(&self, address: Address) -> Option<Bytes>;

	/// Query an account.
	/// Returns the RLP of the account structure or `None` if it doesn't exist.
	fn account(&self, address: Address) -> Option<Bytes>;

	/// Query an account's storage by key.
	/// Returns `None` if it doesn't exist.
	fn storage(&self, address: Address, key: H256) -> Option<H256>;

	/// Commit an account's code.
	fn commit_code(&mut self, address: Address, code: Bytes);

	/// Commit an account's storage.
	///
	/// The iterable provided is a list of storage keys and entries to commit.
	/// If an entry is equal to the zero hash, this means that the key should
	/// be removed.
	fn commit_storage<I>(&mut self, root: &mut H256, iterable: I)
		where I: IntoIterator<Item=(H256, H256)>;
}