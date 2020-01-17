// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Manages local node data: pending local transactions, sync security level

use std::io;
use std::sync::Arc;
use std::time::Duration;

use common_types::{
	BlockNumber,
	transaction::{
		SignedTransaction, PendingTransaction, UnverifiedTransaction,
		Condition as TransactionCondition
	}
};
use ethcore_io::{IoHandler, TimerToken, IoContext};
use kvdb::KeyValueDB;
use log::{debug, trace, warn};
use rlp::Rlp;
use serde_derive::{Serialize, Deserialize};
use serde_json;

const LOCAL_TRANSACTIONS_KEY: &'static [u8] = &*b"LOCAL_TXS";

const UPDATE_TIMER: TimerToken = 0;
const UPDATE_TIMEOUT: Duration = Duration::from_secs(15 * 60); // once every 15 minutes.

#[derive(Serialize, Deserialize)]
enum Condition {
	Number(BlockNumber),
	Timestamp(u64),
}

impl From<TransactionCondition> for Condition {
	fn from(cond: TransactionCondition) -> Self {
		match cond {
			TransactionCondition::Number(num) => Condition::Number(num),
			TransactionCondition::Timestamp(tm) => Condition::Timestamp(tm),
		}
	}
}

impl Into<TransactionCondition> for Condition {
	fn into(self) -> TransactionCondition {
		match self {
			Condition::Number(num) => TransactionCondition::Number(num),
			Condition::Timestamp(tm) => TransactionCondition::Timestamp(tm),
		}
	}
}

#[derive(Serialize, Deserialize)]
struct TransactionEntry {
	rlp_bytes: Vec<u8>,
	condition: Option<Condition>,
}

impl TransactionEntry {
	fn into_pending(self) -> Option<PendingTransaction> {
		let tx: UnverifiedTransaction = match Rlp::new(&self.rlp_bytes).as_val() {
			Err(e) => {
				warn!(target: "local_store", "Invalid persistent transaction stored: {}", e);
				return None
			}
			Ok(tx) => tx,
		};

		let hash = tx.hash();
		match SignedTransaction::new(tx) {
			Ok(tx) => Some(PendingTransaction::new(tx, self.condition.map(Into::into))),
			Err(_) => {
				warn!(target: "local_store", "Bad signature on persistent transaction: {}", hash);
				return None
			}
		}
	}
}

impl From<PendingTransaction> for TransactionEntry {
	fn from(pending: PendingTransaction) -> Self {
		TransactionEntry {
			rlp_bytes: ::rlp::encode(&pending.transaction),
			condition: pending.condition.map(Into::into),
		}
	}
}

/// Something which can provide information about the local node.
pub trait NodeInfo: Send + Sync {
	/// Get all pending transactions of local origin.
	fn pending_transactions(&self) -> Vec<PendingTransaction>;
}

/// Create a new local data store, given a database, a column to write to, and a node.
/// Attempts to read data out of the store, and move it into the node.
pub fn create<T: NodeInfo>(db: Arc<dyn KeyValueDB>, col: u32, node: T) -> LocalDataStore<T> {
	LocalDataStore {
		db,
		col,
		node,
	}
}

/// Manages local node data.
///
/// In specific, this will be used to store things like unpropagated local transactions
/// and the node security level.
pub struct LocalDataStore<T: NodeInfo> {
	db: Arc<dyn KeyValueDB>,
	col: u32,
	node: T,
}

impl<T: NodeInfo> LocalDataStore<T> {
	/// Attempt to read pending transactions out of the local store.
	pub fn pending_transactions(&self) -> io::Result<Vec<PendingTransaction>> {
		if let Some(val) = self.db.get(self.col, LOCAL_TRANSACTIONS_KEY)? {
			let local_txs: Vec<_> = serde_json::from_slice::<Vec<TransactionEntry>>(&val)?
				.into_iter()
				.filter_map(TransactionEntry::into_pending)
				.collect();

			Ok(local_txs)
		} else {
			Ok(Vec::new())
		}
	}

	/// Update the entries in the database.
	pub fn update(&self) -> io::Result<()> {
		trace!(target: "local_store", "Updating local store entries.");

		let local_entries: Vec<TransactionEntry> = self.node.pending_transactions()
			.into_iter()
			.map(Into::into)
			.collect();

		self.write_txs(&local_entries)
	}

	/// Clear data in this column.
	pub fn clear(&self) -> io::Result<()> {
		trace!(target: "local_store", "Clearing local store entries.");

		self.write_txs(&[])
	}

	// helper for writing a vector of transaction entries to disk.
	fn write_txs(&self, txs: &[TransactionEntry]) -> io::Result<()> {
		let mut batch = self.db.transaction();

		let local_json = serde_json::to_value(txs)?;
		let json_str = format!("{}", local_json);

		batch.put_vec(self.col, LOCAL_TRANSACTIONS_KEY, json_str.into_bytes());
		self.db.write(batch)
	}
}

impl<T: NodeInfo, M: Send + Sync + 'static> IoHandler<M> for LocalDataStore<T> {
	fn initialize(&self, io: &IoContext<M>) {
		if let Err(e) = io.register_timer(UPDATE_TIMER, UPDATE_TIMEOUT) {
			warn!(target: "local_store", "Error registering local store update timer: {}", e);
		}
	}

	fn timeout(&self, _io: &IoContext<M>, timer: TimerToken) {
		if let UPDATE_TIMER = timer {
			if let Err(e) = self.update() {
				debug!(target: "local_store", "Error updating local store: {}", e);
			}
		}
	}
}

impl<T: NodeInfo> Drop for LocalDataStore<T> {
	fn drop(&mut self) {
		debug!(target: "local_store", "Updating node data store on shutdown.");

		let _ = self.update();
	}
}

#[cfg(test)]
mod tests {
	use super::NodeInfo;

	use std::sync::Arc;
	use common_types::transaction::{Transaction, Condition, PendingTransaction};
	use ethkey::Brain;
	use parity_crypto::publickey::Generator;

	// we want to test: round-trip of good transactions.
	// failure to roundtrip bad transactions (but that it doesn't panic)

	struct Dummy(Vec<PendingTransaction>);
	impl NodeInfo for Dummy {
		fn pending_transactions(&self) -> Vec<PendingTransaction> { self.0.clone() }
	}

	#[test]
	fn twice_empty() {
		let db = Arc::new(::kvdb_memorydb::create(1));

		{
			let store = super::create(db.clone(), 0, Dummy(vec![]));
			assert_eq!(store.pending_transactions().unwrap(), vec![])
		}

		{
			let store = super::create(db.clone(), 0, Dummy(vec![]));
			assert_eq!(store.pending_transactions().unwrap(), vec![])
		}
	}

	#[test]
	fn with_condition() {
		let keypair = Brain::new("abcd".into()).generate().unwrap();
		let transactions: Vec<_> = (0..10u64).map(|nonce| {
			let mut tx = Transaction::default();
			tx.nonce = nonce.into();

			let signed = tx.sign(keypair.secret(), None);
			let condition = match nonce {
				5 => Some(Condition::Number(100_000)),
				_ => None,
			};

			PendingTransaction::new(signed, condition)
		}).collect();

		let db = Arc::new(::kvdb_memorydb::create(1));

		{
			// nothing written yet, will write pending.
			let store = super::create(db.clone(), 0, Dummy(transactions.clone()));
			assert_eq!(store.pending_transactions().unwrap(), vec![])
		}
		{
			// pending written, will write nothing.
			let store = super::create(db.clone(), 0, Dummy(vec![]));
			assert_eq!(store.pending_transactions().unwrap(), transactions)
		}
		{
			// pending removed, will write nothing.
			let store = super::create(db.clone(), 0, Dummy(vec![]));
			assert_eq!(store.pending_transactions().unwrap(), vec![])
		}
	}

	#[test]
	fn skips_bad_transactions() {
		let keypair = Brain::new("abcd".into()).generate().unwrap();
		let mut transactions: Vec<_> = (0..10u64).map(|nonce| {
			let mut tx = Transaction::default();
			tx.nonce = nonce.into();

			let signed = tx.sign(keypair.secret(), None);

			PendingTransaction::new(signed, None)
		}).collect();

		transactions.push({
			let mut tx = Transaction::default();
			tx.nonce = 10.into();

			let signed = tx.fake_sign(Default::default());
			PendingTransaction::new(signed, None)
		});

		let db = Arc::new(::kvdb_memorydb::create(1));
		{
			// nothing written, will write bad.
			let store = super::create(db.clone(), 0, Dummy(transactions.clone()));
			assert_eq!(store.pending_transactions().unwrap(), vec![])
		}
		{
			// try to load transactions. The last transaction, which is invalid, will be skipped.
			let store = super::create(db.clone(), 0, Dummy(vec![]));
			let loaded = store.pending_transactions().unwrap();
			transactions.pop();
			assert_eq!(loaded, transactions);
		}
	}
}
