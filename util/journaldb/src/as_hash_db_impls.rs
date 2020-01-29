// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Impls of the `AsHashDB` upcast trait for all different variants of DB
use hash_db::{HashDB, AsHashDB};
use keccak_hasher::KeccakHasher;

use kvdb::DBValue;

use crate::{
	archivedb::ArchiveDB,
	earlymergedb::EarlyMergeDB,
	overlayrecentdb::OverlayRecentDB,
	refcounteddb::RefCountedDB,
	overlaydb::OverlayDB,
};

impl AsHashDB<KeccakHasher, DBValue> for ArchiveDB {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}

impl AsHashDB<KeccakHasher, DBValue> for EarlyMergeDB {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}

impl AsHashDB<KeccakHasher, DBValue> for OverlayRecentDB {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}

impl AsHashDB<KeccakHasher, DBValue> for RefCountedDB {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}

impl AsHashDB<KeccakHasher, DBValue> for OverlayDB {
	fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> { self }
	fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> { self }
}
