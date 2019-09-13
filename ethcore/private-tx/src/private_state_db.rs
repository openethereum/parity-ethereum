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

use std::sync::Arc;
use ethereum_types::H256;
use bytes::Bytes;
use kvdb::{KeyValueDB, DBTransaction};
use keccak_hasher::KeccakHasher;
use hash_db::Hasher;
use ethcore_db::COL_PRIVATE_TRANSACTIONS_STATE;
use error::Error;

/// Wrapper around local db with private state for sync purposes
pub struct PrivateStateDB {
	db: Arc<dyn KeyValueDB>,
}

impl PrivateStateDB {
	/// Constructs the object
	pub fn new(db: Arc<dyn KeyValueDB>) -> Self {
		PrivateStateDB {
			db,
		}
	}

	/// Returns saved state for the hash
	pub fn state(&self, state_hash: &H256) -> Result<Bytes, Error> {
		trace!(target: "privatetx", "Retrieve private state from db with hash: {:?}", state_hash);
		self.db.get(COL_PRIVATE_TRANSACTIONS_STATE, state_hash.as_bytes())
			.expect("Low-level database error. Some issue with your hard disk?")
			.map(|s| s.to_vec())
			.ok_or(Error::PrivateStateNotFound)
	}

	/// Stores state for the hash
	pub fn save_state(&self, storage: &Bytes) -> Result<H256, Error> {
		let state_hash = self.state_hash(storage)?;
		let mut transaction = DBTransaction::new();
		transaction.put(COL_PRIVATE_TRANSACTIONS_STATE, state_hash.as_bytes(), storage);
		self.db.write(transaction).map_err(|_| Error::DatabaseWriteError)?;
		trace!(target: "privatetx", "Private state saved to db, its hash: {:?}", state_hash);
		Ok(state_hash)
	}

	/// Returns state's hash without committing it to DB
	pub fn state_hash(&self, state: &Bytes) -> Result<H256, Error> {
		Ok(KeccakHasher::hash(state))
	}
}
