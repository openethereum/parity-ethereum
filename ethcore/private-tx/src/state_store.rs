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

use std::collections::{HashMap};
use std::sync::{Arc};
use parking_lot::{RwLock};
use bytes::Bytes;
use ethereum_types::{Address};
use error::{Error, ErrorKind};
use types::transaction::{SignedTransaction};
use private_transactions::{VerifiedPrivateTransaction};

/// Wrapper over storage for the private states
pub struct PrivateStateStore {
	verification_requests: RwLock<Vec<Arc<VerifiedPrivateTransaction>>>,
	creation_requests: RwLock<Vec<SignedTransaction>>,
	temp_offchain_storage: RwLock<HashMap<Address, Bytes>>,
}

impl PrivateStateStore {
	/// Constructs the object
	pub fn new() -> Self {
		PrivateStateStore {
			verification_requests: RwLock::new(Vec::new()),
			creation_requests: RwLock::new(Vec::new()),
			temp_offchain_storage: RwLock::default(),
		}
	}

	/// Returns saved state for the address
	pub fn state(&self, address: &Address) -> Result<Bytes, Error> {
		let offchain_storage = self.temp_offchain_storage.read();
		match offchain_storage.get(address) {
			Some(state) => Ok(state.to_vec()),
			None => bail!(ErrorKind::PrivateStateNotFound),
		}
	}

	/// Stores state for the address
	pub fn save_state(&self, address: &Address, storage: Bytes) {
		let mut offchain_storage = self.temp_offchain_storage.write();
		offchain_storage.insert(*address, storage);
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