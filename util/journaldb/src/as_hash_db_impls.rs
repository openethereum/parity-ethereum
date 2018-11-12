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

//! Impls of the `AsHashDB` upcast trait for all different variants of DB
use hashdb::{HashDB, AsHashDB};
use keccak_hasher::KeccakHasher;
use archivedb::ArchiveDB;
use earlymergedb::EarlyMergeDB;
use overlayrecentdb::OverlayRecentDB;
use refcounteddb::RefCountedDB;
use overlaydb::OverlayDB;
use kvdb::DBValue;

impl AsHashDB<KeccakHasher, DBValue> for ArchiveDB {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }
}

impl AsHashDB<KeccakHasher, DBValue> for EarlyMergeDB {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }
}

impl AsHashDB<KeccakHasher, DBValue> for OverlayRecentDB {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }
}

impl AsHashDB<KeccakHasher, DBValue> for RefCountedDB {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }
}

impl AsHashDB<KeccakHasher, DBValue> for OverlayDB {
	fn as_hashdb(&self) -> &HashDB<KeccakHasher, DBValue> { self }
	fn as_hashdb_mut(&mut self) -> &mut HashDB<KeccakHasher, DBValue> { self }
}
