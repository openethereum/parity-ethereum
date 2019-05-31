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
use kvdb::KeyValueDB;
use types::transaction::SignedTransaction;
use private_transactions::VerifiedPrivateTransaction;
use private_state_db::PrivateStateDB;

/// State of the private state sync
#[derive(Clone, PartialEq)]
pub enum SyncState {
	/// No sync is running
	Idle,
	/// Private state sync is running
	Syncing,
}

/// Wrapper over storage for the private states
pub struct PrivateStateStorage {
	verification_requests: RwLock<Vec<Arc<VerifiedPrivateTransaction>>>,
	creation_requests: RwLock<Vec<SignedTransaction>>,
	private_state_db: Arc<PrivateStateDB>,
	sync_state: RwLock<SyncState>,
	syncing_hashes: RwLock<Vec<H256>>,
}

impl PrivateStateStorage {
	/// Constructs the object
	pub fn new(db: Arc<KeyValueDB>) -> Self {
		PrivateStateStorage {
			verification_requests: RwLock::new(Vec::new()),
			creation_requests: RwLock::new(Vec::new()),
			private_state_db: Arc::new(PrivateStateDB::new(db)),
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

	/// Signals that corresponding private state retrieved and added into the local db
	pub fn state_sync_completed(&self, synced_state_hash: &H256) {
		let mut syncing_hashes = self.syncing_hashes.write();
		if let Some(index) = syncing_hashes.iter().position(|h| h == synced_state_hash) {
			syncing_hashes.remove(index);
		}
		if syncing_hashes.is_empty() {
			// All states were downloaded
			*self.sync_state.write() = SyncState::Idle;
		}
	}

	/// Returns underlying DB
	pub fn private_state_db(&self) -> Arc<PrivateStateDB> {
		self.private_state_db.clone()
	}

	/// Stores verification request for the later verification
	pub fn add_verification_request(&self, transaction: Arc<VerifiedPrivateTransaction>) {
		let mut verification_requests = self.verification_requests.write();
		verification_requests.push(transaction);
	}

	/// Drains all verification requests to process
	pub fn drain_verification_queue(&self) -> Vec<Arc<VerifiedPrivateTransaction>> {
		let mut requests_queue = self.verification_requests.write();
		let requests = requests_queue.drain(..).collect::<Vec<_>>();
		requests
	}

	/// Stores creation request for the later creation
	pub fn add_creation_request(&self, transaction: SignedTransaction) {
		let mut creation_requests = self.creation_requests.write();
		creation_requests.push(transaction);
	}

	/// Drains all creation requests to process
	pub fn drain_creation_queue(&self) -> Vec<SignedTransaction> {
		let mut requests_queue = self.creation_requests.write();
		let requests = requests_queue.drain(..).collect::<Vec<_>>();
		requests
	}
}