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

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Instant, Duration};
use parking_lot::RwLock;
use ethereum_types::H256;
use kvdb::KeyValueDB;
use types::transaction::SignedTransaction;
use private_transactions::VerifiedPrivateTransaction;
use private_state_db::PrivateStateDB;
use log::Logging;

/// Max duration of retrieving state (in ms)
const MAX_REQUEST_SESSION_DURATION: u64 = 120 * 1000;

struct HashRequestSession {
	hash: H256,
	expiration_time: Instant,
}

/// Type of the stored reques
pub enum RequestType {
	/// Verification of private transaction
	Verification(Arc<VerifiedPrivateTransaction>),
	/// Creation of the private transaction
	Creation(SignedTransaction),
}

#[derive(Clone, PartialEq)]
enum RequestState {
	Syncing,
	Ready,
}

struct StateRequest {
	request_type: RequestType,
	request_hashes: HashSet<H256>,
	state: RequestState,
}

/// Wrapper over storage for the private states
pub struct PrivateStateStorage {
	private_state_db: Arc<PrivateStateDB>,
	requests: RwLock<Vec<StateRequest>>,
	syncing_hashes: RwLock<Vec<HashRequestSession>>,
	logging: Option<Arc<Logging>>,
}

impl PrivateStateStorage {
	/// Constructs the object
	pub fn new(db: Arc<KeyValueDB>, logging: Option<Arc<Logging>>) -> Self {
		PrivateStateStorage {
			private_state_db: Arc::new(PrivateStateDB::new(db)),
			requests: RwLock::new(Vec::new()),
			syncing_hashes: RwLock::default(),
			logging,
		}
	}

	/// Checks if ready for processing requests exist in queue
	pub fn requests_ready(&self) -> bool {
		let requests = self.requests.read();
		requests.iter().find(|r| r.state == RequestState::Ready).is_some()
	}

	/// Signals that corresponding private state retrieved and added into the local db
	pub fn state_sync_completed(&self, synced_state_hash: &H256) {
		let mut syncing_hashes = self.syncing_hashes.write();
		if let Some(index) = syncing_hashes.iter().position(|h| h.hash == *synced_state_hash) {
			syncing_hashes.remove(index);
		}
		self.mark_hash_ready(synced_state_hash);
	}

	/// Returns underlying DB
	pub fn private_state_db(&self) -> Arc<PrivateStateDB> {
		self.private_state_db.clone()
	}

	/// Store a request for state's sync and later processing, returns new hashes, which sync is required
	pub fn add_request(&self, request_type: RequestType, request_hashes: HashSet<H256>) -> Vec<H256> {
		let request = StateRequest {
			request_type: request_type,
			request_hashes: request_hashes.clone(),
			state: RequestState::Syncing,
		};
		let mut requests = self.requests.write();
		requests.push(request);
		let mut new_hashes = Vec::new();
		for hash in request_hashes {
			let mut hashes = self.syncing_hashes.write();
			if hashes.iter().find(|&h| h.hash == hash).is_none() {
				let hash_session = HashRequestSession {
					hash,
					expiration_time: Instant::now() + Duration::from_millis(MAX_REQUEST_SESSION_DURATION),
				};
				hashes.push(hash_session);
				new_hashes.push(hash);
			}
		}
		new_hashes
	}

	/// Drains ready requests to process
	pub fn drain_ready_requests(&self) -> Vec<RequestType> {
		let mut requests_queue = self.requests.write();
		let mut drained = Vec::new();
		let mut i = 0;
		while i != requests_queue.len() {
			if requests_queue[i].state == RequestState::Ready {
				let request = requests_queue.remove(i);
				drained.push(request.request_type);
			} else {
				i += 1;
			}
		}
		drained
	}

	/// State retrieval timer's tick
	pub fn tick(&self) {
		let mut syncing_hashes = self.syncing_hashes.write();
		for hash in syncing_hashes.iter() {
			if hash.expiration_time >= Instant::now() {
				self.mark_hash_stale(&hash.hash);
			}
		}
		syncing_hashes.retain(|hash| hash.expiration_time < Instant::now());
	}

	fn mark_hash_ready(&self, ready_hash: &H256) {
		let mut requests = self.requests.write();
		for request in requests.iter_mut() {
			request.request_hashes.remove(ready_hash);
			if request.request_hashes.is_empty() && request.state == RequestState::Syncing {
				request.state = RequestState::Ready;
			}
		}
	}

	fn mark_hash_stale(&self, stale_hash: &H256) {
		let mut requests = self.requests.write();
		requests.retain(|request| {
			let mut delete_request = false;
			if request.request_hashes.contains(stale_hash) {
				let tx_hash;
				match &request.request_type {
					RequestType::Verification(transaction) => {
						tx_hash = transaction.transaction_hash;
					}
					RequestType::Creation(transaction) => {
						tx_hash = transaction.hash();
						if let Some(ref logging) = self.logging {
							logging.private_state_sync_failed(&tx_hash);
						}
					}
				}
				trace!(target: "privatetx", "Private state request for {:?} staled due to timeout", &tx_hash);
				delete_request = true;
			}
			!delete_request
		});
	}
}