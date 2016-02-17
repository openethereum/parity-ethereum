// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::vec::Vec;
use std::cmp::{Ordering};
use std::collections::{HashMap, BTreeSet};
use std::sync::{RwLock, Arc};
use util::executor::*;
use util::uint::U256;
use util::hash::Address;
use ethcore::transaction::*;

pub type VerifyTxTask = ClosureTask<(), ()>;

struct VerifiedTransaction {
	tx: SignedTransaction,
	nonce_height: U256
}

impl VerifiedTransaction {
	pub fn new(tx: SignedTransaction, nonce_height: U256) -> VerifiedTransaction {
		VerifiedTransaction {
			tx: tx,
			nonce_height: nonce_height
		}
	}
}
trivial_ordering!(VerifiedTransaction by |a: &VerifiedTransaction, b: &VerifiedTransaction| {
	if a.nonce_height == b.nonce_height {
		a.tx.gas_price.cmp(&b.tx.gas_price)
	} else {
		a.nonce_height.cmp(&b.nonce_height)
	}
});

pub struct TxQueueStats {
	pub pending: usize,
	pub queued: usize,
}

pub struct TxQueue<'a> {
	executor: &'a Executor<VerifyTxTask>,
	limit: usize,
	current: Arc<RwLock<BTreeSet<VerifiedTransaction>>>,
	known_nonces: Arc<RwLock<HashMap<Address, U256>>>,
}

impl<'a> TxQueue<'a> {
	/// Creates new instance of this Queue
	pub fn new(executor: &'a Executor<VerifyTxTask>) -> Self {
		let limit = 1024;
		let current = BTreeSet::new();
		let nonces = HashMap::new();

		TxQueue {
			executor: executor,
			limit: limit,
			current: Arc::new(RwLock::new(current)),
			known_nonces: Arc::new(RwLock::new(nonces)),
		}
	}

	/// Returns current stats for this queue
	pub fn stats(&self) -> TxQueueStats {
		TxQueueStats {
			pending: 0,
			queued: self.executor.queued()
		}
	}

	/// Add signed transaction to queue to be verified and imported
	pub fn add(&mut self, tx: SignedTransaction) {
		let current = self.current.clone();
		let known_nonces = self.known_nonces.clone();

		self.executor.execute(Executors::task(move || {
			let mut queue = current.write().unwrap();
			let nonces = known_nonces.read().unwrap();

			let address = tx.sender().unwrap();
			let nonce_height = nonces
				.get(&address)
				.map_or_else(|| tx.nonce, |nonce| tx.nonce - nonce.clone());
			queue.insert(VerifiedTransaction::new(tx, nonce_height));
			Ok(())
		})).fire();
	}

	/// Returns top transactions from the queue
	pub fn top_transactions(&self, size: usize) -> Vec<SignedTransaction> {
		let current = self.current.read().unwrap();
		current.iter().take(size).map(|t| t.tx.clone()).collect()
	}
}

#[cfg(test)]
mod test {
	extern crate rustc_serialize;
	use self::rustc_serialize::hex::FromHex;
	use std::ops::Deref;

	use util::crypto::KeyPair;
	use util::executor::Executors;
	use util::uint::{U256, Uint};
	use ethcore::transaction::*;
	use super::*;

	fn new_tx() -> SignedTransaction {
		let keypair = KeyPair::create().unwrap();
		Transaction {
			action: Action::Create,
			value: U256::from(100),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::one(),
			nonce: U256::zero()
		}.sign(&keypair.secret())
	}

	#[test]
	fn should_enqueue_tx() {
		// given
		let exec = Executors::manual();
		let mut txq = TxQueue::new(&exec);
		let tx = new_tx();

		// when
		txq.add(tx);

		// then
		let stats = txq.stats();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.queued, 1);
	}

	#[test]
	fn should_import_txs_from_same_sender() {
		// given
		let exec = Executors::same_thread();
		let mut txq = TxQueue::new(&exec);
		let keypair = KeyPair::create().unwrap();

		let mut unsigned_tx = new_tx().deref().clone();
		let tx = unsigned_tx.clone().sign(&keypair.secret());
		unsigned_tx.nonce = U256::from(1);
		let tx2 = unsigned_tx.clone().sign(&keypair.secret());


		// when
		txq.add(tx2.clone());
		txq.add(tx.clone());

		// then
		let top = txq.top_transactions(5);
		assert_eq!(top.len(), 2);
		assert_eq!(top[0], tx);
		assert_eq!(top[1], tx2);
	}

}
