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

//! Notifier for new transaction hashes.

use std::fmt;
use std::sync::Arc;

use ethereum_types::H256;
use futures::sync::mpsc;
use txpool::{self, VerifiedTransaction};

use pool::VerifiedTransaction as Transaction;
use pool::TxStatus;

/// Transaction pool logger.
#[derive(Default, Debug)]
pub struct Logger;

impl txpool::Listener<Transaction> for Logger {
	fn added(&mut self, tx: &Arc<Transaction>, old: Option<&Arc<Transaction>>) {
		debug!(target: "txqueue", "[{:?}] Added to the pool.", tx.hash());
		debug!(
			target: "txqueue",
			"[{hash:?}] Sender: {sender}, nonce: {nonce}, gasPrice: {gas_price}, gas: {gas}, value: {value}, dataLen: {data}))",
			hash = tx.hash(),
			sender = tx.sender(),
			nonce = tx.signed().nonce,
			gas_price = tx.signed().gas_price,
			gas = tx.signed().gas,
			value = tx.signed().value,
			data = tx.signed().data.len(),
		);

		if let Some(old) = old {
			debug!(target: "txqueue", "[{:?}] Dropped. Replaced by [{:?}]", old.hash(), tx.hash());
		}
	}

	fn rejected<H: fmt::Debug + fmt::LowerHex>(&mut self, _tx: &Arc<Transaction>, reason: &txpool::Error<H>) {
		trace!(target: "txqueue", "Rejected {}.", reason);
	}

	fn dropped(&mut self, tx: &Arc<Transaction>, new: Option<&Transaction>) {
		match new {
			Some(new) => debug!(target: "txqueue", "[{:?}] Pushed out by [{:?}]", tx.hash(), new.hash()),
			None => debug!(target: "txqueue", "[{:?}] Dropped.", tx.hash()),
		}
	}

	fn invalid(&mut self, tx: &Arc<Transaction>) {
		debug!(target: "txqueue", "[{:?}] Marked as invalid by executor.", tx.hash());
	}

	fn canceled(&mut self, tx: &Arc<Transaction>) {
		debug!(target: "txqueue", "[{:?}] Canceled by the user.", tx.hash());
	}

	fn culled(&mut self, tx: &Arc<Transaction>) {
		debug!(target: "txqueue", "[{:?}] Culled or mined.", tx.hash());
	}
}

/// Transactions pool notifier
#[derive(Default)]
pub struct TransactionsPoolNotifier {
	full_listeners: Vec<mpsc::UnboundedSender<Arc<Vec<(H256, TxStatus)>>>>,
	pending_listeners: Vec<mpsc::UnboundedSender<Arc<Vec<H256>>>>,
	tx_statuses: Vec<(H256, TxStatus)>,
}

impl TransactionsPoolNotifier {
	/// Add new full listener to receive notifications.
	pub fn add_full_listener(&mut self, f: mpsc::UnboundedSender<Arc<Vec<(H256, TxStatus)>>>) {
		self.full_listeners.push(f);
	}

	/// Add new pending listener to receive notifications.
	pub fn add_pending_listener(&mut self, f: mpsc::UnboundedSender<Arc<Vec<H256>>>) {
		self.pending_listeners.push(f);
	}

	/// Notify listeners about all currently transactions.
	pub fn notify(&mut self) {
		if self.tx_statuses.is_empty() {
			return;
		}

		let to_pending_send: Arc<Vec<H256>> = Arc::new(
			self.tx_statuses.clone()
				.into_iter()
				.map(|(hash, _)| hash)
				.collect()
		);
		self.pending_listeners.retain(|listener| {
			listener.unbounded_send(to_pending_send.clone()).is_ok()
		});

		let to_full_send = Arc::new(
			std::mem::replace(&mut self.tx_statuses, Vec::new())
		);
		self.full_listeners
			.retain(|listener| {
				listener.unbounded_send(to_full_send.clone()).is_ok()
			});
	}
}

impl fmt::Debug for TransactionsPoolNotifier {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("TransactionsPoolNotifier")
			.field("full_listeners", &self.full_listeners.len())
			.field("pending_listeners", &self.pending_listeners.len())
			.finish()
	}
}

impl txpool::Listener<Transaction> for TransactionsPoolNotifier {
	fn added(&mut self, tx: &Arc<Transaction>, _old: Option<&Arc<Transaction>>) {
		self.tx_statuses.push((tx.hash.clone(), TxStatus::Added));
	}

	fn rejected<H: fmt::Debug + fmt::LowerHex>(&mut self, tx: &Arc<Transaction>, _reason: &txpool::Error<H>) {
		self.tx_statuses.push((tx.hash.clone(), TxStatus::Rejected));
	}

	fn dropped(&mut self, tx: &Arc<Transaction>, _new: Option<&Transaction>) {
		self.tx_statuses.push((tx.hash.clone(), TxStatus::Dropped));
	}

	fn invalid(&mut self, tx: &Arc<Transaction>) {
		self.tx_statuses.push((tx.hash.clone(), TxStatus::Invalid));
	}

	fn canceled(&mut self, tx: &Arc<Transaction>) {
		self.tx_statuses.push((tx.hash.clone(), TxStatus::Canceled));
	}

	fn culled(&mut self, tx: &Arc<Transaction>) {
		self.tx_statuses.push((tx.hash.clone(), TxStatus::Culled));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use types::transaction;
	use txpool::Listener;
	use futures::{Stream, Future};
	use ethereum_types::Address;

	#[test]
	fn should_notify_listeners() {
		// given
		let (full_sender, full_receiver) = mpsc::unbounded();
		let (pending_sender, pending_receiver) = mpsc::unbounded();

		let mut tx_listener = TransactionsPoolNotifier::default();
		tx_listener.add_full_listener(full_sender);
		tx_listener.add_pending_listener(pending_sender);

		// when
		let tx = new_tx();
		tx_listener.added(&tx, None);

		// then
		tx_listener.notify();
		let (full_res , _full_receiver)= full_receiver.into_future().wait().unwrap();
		let (pending_res , _pending_receiver)= pending_receiver.into_future().wait().unwrap();
		assert_eq!(
			full_res,
			Some(Arc::new(vec![(serde_json::from_str::<H256>("\"0x13aff4201ac1dc49daf6a7cf07b558ed956511acbaabf9502bdacc353953766d\"").unwrap(), TxStatus::Added)]))
		);
		assert_eq!(
			pending_res,
			Some(Arc::new(vec![serde_json::from_str::<H256>("\"0x13aff4201ac1dc49daf6a7cf07b558ed956511acbaabf9502bdacc353953766d\"").unwrap()]))
		);
	}

	fn new_tx() -> Arc<Transaction> {
		let signed = transaction::Transaction {
			action: transaction::Action::Create,
			data: vec![1, 2, 3],
			nonce: 5.into(),
			gas: 21_000.into(),
			gas_price: 5.into(),
			value: 0.into(),
		}.fake_sign(Address::from_low_u64_be(5));

		Arc::new(Transaction::from_pending_block_transaction(signed))
	}
}
