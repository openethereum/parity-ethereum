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
use std::collections::hash_map::Entry;
use std::sync::Arc;
use parking_lot::RwLock;
use bytes::Bytes;
use ethereum_types::{Address, U256};
use error::{Error, ErrorKind};
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

/// Private state saved in the storage
#[derive(Clone)]
pub struct StoredPrivateState {
	/// State data
	pub data: Bytes,
	/// Corresponding nonce
	pub nonce: U256,
}

/// Wrapper over storage for the private states
pub struct PrivateStateStore {
	verification_requests: RwLock<Vec<Arc<VerifiedPrivateTransaction>>>,
	creation_requests: RwLock<Vec<SignedTransaction>>,
	temp_offchain_storage: RwLock<HashMap<Address, StoredPrivateState>>,
	sync_state: RwLock<SyncState>,
	syncing_private_states: RwLock<HashMap<Address, U256>>,
}

impl PrivateStateStore {
	/// Constructs the object
	pub fn new() -> Self {
		PrivateStateStore {
			verification_requests: RwLock::new(Vec::new()),
			creation_requests: RwLock::new(Vec::new()),
			temp_offchain_storage: RwLock::default(),
			sync_state: RwLock::new(SyncState::Idle),
			syncing_private_states: RwLock::default(),
		}
	}

	/// Current sync state
	pub fn current_sync_state(&self) -> SyncState {
		(*self.sync_state.read()).clone()
	}

	/// Adds information about states being synced now
	pub fn start_states_sync(&self, states_to_sync: &Vec<(Address, U256)>) -> Vec<Address> {
		*self.sync_state.write() = SyncState::Syncing;
		let mut addresses_to_sync = Vec::new();
		for state in states_to_sync {
			if let Some(old_nonce) = self.syncing_private_states.write().insert(state.0, state.1) {
				if old_nonce < state.1 {
					// Required nonce for the private contract is greater, when requested before, so it needs to be requested again
					addresses_to_sync.push(state.0);
				}
			} else {
				// State for this contract was not requested yet
				addresses_to_sync.push(state.0);
			}
		}
		addresses_to_sync
	}

	pub fn state_sync_completed(&self, synced_states: &Vec<(Address, U256)>) -> Vec<Address> {
		let mut syncing_states = self.syncing_private_states.write();
		let mut addresses_to_store = Vec::new();
		for state in synced_states {
			let synced_state = syncing_states.entry(state.0);
			match synced_state {
				Entry::Occupied(syncing_state) => {
					if *syncing_state.get() <= state.1 {
						// Received private state is good (nonce as requested or newer), store it
						addresses_to_store.push(state.0);
						syncing_state.remove_entry();
					}
				}
				Entry::Vacant(_) => {
					warn!(target: "privatetx", "Synced state was not stored for syncing");
				}
			}
		}
		if syncing_states.is_empty() {
			// All states were downloaded
			*self.sync_state.write() = SyncState::Idle;
		}
		addresses_to_store
	}

	/// Returns saved state for the address
	pub fn state(&self, address: &Address) -> Result<StoredPrivateState, Error> {
		let offchain_storage = self.temp_offchain_storage.read();
		match offchain_storage.get(address) {
			Some(state) => Ok(state.clone()),
			None => bail!(ErrorKind::PrivateStateNotFound),
		}
	}

	/// Stores state for the address
	pub fn save_state(&self, address: &Address, storage: Bytes, nonce: U256) {
		let mut offchain_storage = self.temp_offchain_storage.write();
		let state_to_store = StoredPrivateState {
			data: storage,
			nonce,
		};
		offchain_storage.insert(*address, state_to_store);
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