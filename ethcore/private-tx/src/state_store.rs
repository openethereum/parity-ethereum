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

use std::collections::HashMap;
use std::sync::Arc;
use hash::keccak;
use parking_lot::RwLock;
use bytes::Bytes;
use ethcore_db::COL_PRIVATE_TRANSACTIONS_STATE;
use ethereum_types::H256;
use journaldb::overlaydb::OverlayDB;
use kvdb::KeyValueDB;
use error::Error;
use types::transaction::SignedTransaction;
use private_transactions::VerifiedPrivateTransaction;

/// State of the private state sync
#[derive(Clone)]
pub enum SyncState {
	/// No sync is running
	Idle,
	/// Private state sync is running
	Syncing,
}

/// Wrapper over storage for the private states
pub struct PrivateStateStore {
	verification_requests: RwLock<Vec<Arc<VerifiedPrivateTransaction>>>,
	creation_requests: RwLock<Vec<SignedTransaction>>,
	temp_offchain_storage: RwLock<HashMap<H256, Bytes>>,
	private_state: RwLock<OverlayDB>,
	db: Arc<KeyValueDB>,
	sync_state: RwLock<SyncState>,
	syncing_hashes: RwLock<Vec<H256>>,
}

impl PrivateStateStore {
	/// Constructs the object
	pub fn new(db: Arc<KeyValueDB>) -> Self {
		PrivateStateStore {
			verification_requests: RwLock::new(Vec::new()),
			creation_requests: RwLock::new(Vec::new()),
			temp_offchain_storage: RwLock::default(),
			private_state: RwLock::new(OverlayDB::new(db.clone(), COL_PRIVATE_TRANSACTIONS_STATE)),
			db,
			sync_state: RwLock::new(SyncState::Idle),
			syncing_hashes: RwLock::default(),
		}
	}

	/// Current sync state
	pub fn current_sync_state(&self) -> SyncState {
		(*self.sync_state.read()).clone()
	}

	/// Adds information about states being synced now
	pub fn start_states_sync(&self, hashes_to_sync: &Vec<H256>) -> Vec<H256> {
		*self.sync_state.write() = SyncState::Syncing;
		let mut new_hashes = Vec::new();
		for hash in hashes_to_sync {
			let mut hashes = self.syncing_hashes.write();
			if hashes.iter().find(|&h| h == hash).is_none() {
				hashes.push(*hash);
				new_hashes.push(*hash);
			}
		}
		new_hashes
	}

	pub fn state_sync_completed(&self, synced_states_hashes: &Vec<H256>) {
		let mut syncing_hashes = self.syncing_hashes.write();
		for hash in synced_states_hashes {
			if let Some(index) = syncing_hashes.iter().position(|h| h == hash) {
				syncing_hashes.remove(index);
			}
		}
		if syncing_hashes.is_empty() {
			// All states were downloaded
			*self.sync_state.write() = SyncState::Idle;
		}
	}

	/// Returns saved state for the address
	pub fn state(&self, state_hash: &H256) -> Result<Bytes, Error> {
		let offchain_storage = self.temp_offchain_storage.read();
		match offchain_storage.get(state_hash) {
			Some(state) => Ok(state.clone()),
			None => Err(Error::PrivateStateNotFound),
		}
	}

	/// Stores state for the address
	pub fn save_state(&self, storage: Bytes) {
		let mut offchain_storage = self.temp_offchain_storage.write();
		let state_hash = keccak(&storage);
		offchain_storage.insert(state_hash, storage);
	}

	/// Stores verification request for the later verification
	pub fn add_verification_request(&self, transaction: Arc<VerifiedPrivateTransaction>) {
		let mut verification_requests = self.verification_requests.write();
		verification_requests.push(transaction);
	}

	/// Stores creation request for the later creation
	pub fn add_creation_request(&self, transaction: SignedTransaction) {
		let mut creation_requests = self.creation_requests.write();
		creation_requests.push(transaction);
	}
}