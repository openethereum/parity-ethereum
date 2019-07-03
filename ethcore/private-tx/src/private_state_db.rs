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
use parking_lot::RwLock;
use ethereum_types::H256;
use bytes::Bytes;
use journaldb::overlaydb::OverlayDB;
use kvdb::{KeyValueDB, DBTransaction};
use hash_db::{HashDB, EMPTY_PREFIX};
use ethcore_db::COL_PRIVATE_TRANSACTIONS_STATE;
use error::Error;

/// Wrapper around local db with private state for sync purposes
pub struct PrivateStateDB {
	private_state: RwLock<OverlayDB>,
	db: Arc<KeyValueDB>,
}

impl PrivateStateDB {
	/// Constructs the object
	pub fn new(db: Arc<KeyValueDB>) -> Self {
		PrivateStateDB {
			private_state: RwLock::new(OverlayDB::new(db.clone(), COL_PRIVATE_TRANSACTIONS_STATE)),
			db,
		}
	}

	/// Returns saved state for the hash
	pub fn state(&self, state_hash: &H256) -> Result<Bytes, Error> {
		let private_state = self.private_state.read();
		trace!(target: "privatetx", "Retrieve private state from db with hash: {:?}", state_hash);
		private_state.get(state_hash, EMPTY_PREFIX).map(|s| s.to_vec()).ok_or(Error::PrivateStateNotFound)
	}

	/// Stores state for the hash
	pub fn save_state(&self, storage: &Bytes) -> Result<H256, Error> {
		let mut private_state = self.private_state.write();
		let state_hash = private_state.insert(storage, EMPTY_PREFIX);
		let mut transaction = DBTransaction::new();
		private_state.commit_to_batch(&mut transaction)?;
		self.db.write(transaction).map_err(|_| Error::DatabaseWriteError)?;
		trace!(target: "privatetx", "Private state saved to db, its hash: {:?}", state_hash);
		Ok(state_hash)
	}
}