// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Private transactions module.

pub mod private_transactions;

pub use self::private_transactions::PrivateTransactions;

use std::sync::{Arc, Weak};
use client::{ChainNotify, ChainMessageType};
use transaction::UnverifiedTransaction;
use error::Error as EthcoreError;
use rlp::UntrustedRlp;
use parking_lot::{Mutex, RwLock};
use bytes::Bytes;

/// Manager of private transactions
pub struct Provider {
	notify: RwLock<Vec<Weak<ChainNotify>>>,
	private_transactions: Mutex<PrivateTransactions>,
}

impl Provider {
	/// Create a new provider.
	pub fn new() -> Self {
		Provider {
			notify: RwLock::new(Vec::new()),
			private_transactions: Mutex::new(PrivateTransactions::new()),
		}
	}

	/// Adds an actor to be notified on certain events
	pub fn add_notify(&self, target: Arc<ChainNotify>) {
		self.notify.write().push(Arc::downgrade(&target));
	}

	fn notify<F>(&self, f: F) where F: Fn(&ChainNotify) {
		for np in self.notify.read().iter() {
			if let Some(n) = np.upgrade() {
				f(&*n);
			}
		}
	}

	/// Add private transaction into the store
	pub fn import_private_transaction(&self, rlp: &[u8], peer_id: usize) -> Result<(), EthcoreError> {
		let tx: UnverifiedTransaction = UntrustedRlp::new(rlp).as_val()?;
		// TODO: notify engines about private transactions
		self.private_transactions.lock().import_transaction(tx, peer_id)
	}

	/// Add signed private transaction into the store
	pub fn import_signed_private_transaction(&self, rlp: &[u8], peer_id: usize) -> Result<(), EthcoreError> {
		let tx: UnverifiedTransaction = UntrustedRlp::new(rlp).as_val()?;
		self.private_transactions.lock().import_signed_transaction(tx, peer_id)
	}

	/// Broadcast the private transaction message to chain
	pub fn broadcast_private_transaction(&self, message: Bytes) {
		self.notify(|notify| notify.broadcast(ChainMessageType::PrivateTransaction, message.clone()));
	}

	/// Broadcast signed private transaction message to chain
	pub fn broadcast_signed_private_transaction(&self, message: Bytes) {
		self.notify(|notify| notify.broadcast(ChainMessageType::SignedPrivateTransaction, message.clone()));
	}

	/// Returns the list of private transactions
	pub fn private_transactions(&self) -> Vec<UnverifiedTransaction> {
		self.private_transactions.lock().get_transactions_list()
	}

	/// Returns the list of signed private transactions
	pub fn signed_private_transactions(&self) -> Vec<UnverifiedTransaction> {
		self.private_transactions.lock().get_signed_transactions_list()
	}
}