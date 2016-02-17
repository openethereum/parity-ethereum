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
use std::cmp::{Ordering, max};
use std::collections::BTreeSet;
use std::sync::{RwLock, Arc};
use ethcore::transaction::*;
use util::executor::*;

pub type VerifyTxTask = ClosureTask<(), ()>;

struct VerifiedTransaction {
	tx: SignedTransaction
}

impl VerifiedTransaction {
	pub fn new(tx: SignedTransaction) -> VerifiedTransaction {
		VerifiedTransaction {
			tx: tx
		}
	}
}
trivial_ordering!(VerifiedTransaction by |_a, _b| {
	Ordering::Equal
});

pub struct TxQueueStats {
	pub pending: usize,
	pub queued: usize,
}

pub struct TxQueue<'a> {
	executor: &'a Executor<VerifyTxTask>,
	limit: usize,
	current: Arc<RwLock<BTreeSet<VerifiedTransaction>>>
}

impl<'a> TxQueue<'a> {
	/// Creates new instance of this Queue
	pub fn new(executor: &'a Executor<VerifyTxTask>) -> Self {
		let limit = 1024;
		let current = BTreeSet::new();

		TxQueue {
			executor: executor,
			limit: limit,
			current: Arc::new(RwLock::new(current))
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

		self.executor.execute(Executors::task(move || {
			let mut queue = current.write().unwrap();
			queue.insert(VerifiedTransaction::new(tx));
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
	fn should_import_txs() {
		// given
		let exec = Executors::same_thread();
		let mut txq = TxQueue::new(&exec);
		let tx = new_tx();

		// when
		txq.add(tx.clone());

		// then
		let top = txq.top_transactions(5);
		assert_eq!(top.len(), 1);
		assert_eq!(top[0], tx);
	}

}
