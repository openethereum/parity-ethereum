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

//! Manages local node data: pending local transactions, sync security level

use std::sync::Arc;
use std::fmt;

use ethcore::transaction::{
	SignedTransaction, PendingTransaction, UnverifiedTransaction,
	Condition as TransactionCondition
};
use util::kvdb::KeyValueDB;

extern crate ethcore;
extern crate ethcore_util as util;
extern crate rlp;
extern crate serde_json;
extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

const LOCAL_TRANSACTIONS_KEY: &'static [u8] = &*b"LOCAL_TXS";

/// Errors which can occur while using the local data store.
#[derive(Debug, Clone)]
pub enum Error {
	/// Database errors: these manifest as `String`s.
	Database(String),
	/// JSON errors.
	Json(::serde_json::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::Database(ref val) => write!(f, "{}", val),
			Error::Json(ref err) => write!(f, "{}", err),
		}
	}
}

#[derive(Serialize, Deserialize)]
enum Condition {
	Number(::ethcore::header::BlockNumber),
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
		let tx: UnverifiedTransaction = match ::rlp::decode(&self.rlp_bytes).ok() {
			None => {
				warn!(target: "local_store", "Invalid persistent transaction stored.");
				return None
			}
			Some(tx) => tx,
		};

		let hash = tx.hash();
		match SignedTransaction::new(tx) {
			Ok(tx) => Some(PendingTransaction::new(tx, self.condition.map(Into::into))),
			Err(e) => {
				warn!(target: "local_store", "Bad signature on persistent transaction: {}", hash);
				return None
			}
		}
	}
}

impl From<PendingTransaction> for TransactionEntry {
	fn from(pending: PendingTransaction) -> Self {
		TransactionEntry {
			rlp_bytes: ::rlp::encode(&pending.transaction).to_vec(),
			condition: pending.condition.into(),
		}
	}
}

/// Something which encompasses the exact node-like status for which we store data.
pub trait NodeLike {
	/// Get all pending transactions of local origin.
	fn local_pending_transactions(&self) -> Vec<PendingTransaction>;

	/// Import stored transactions.
	fn import_stored_transactions(&self, Vec<PendingTransaction>);
}

/// Manages local node data.
///
/// In specific, this will be used to store things like unpropagated local transactions
/// and the node security level.
pub struct LocalDataStore<T> {
	db: Arc<KeyValueDB>,
	col: Option<u32>,
	node: T,
}

impl<T: NodeLike> LocalDataStore<T> {
	/// Create a new local data store, given a database, a column to write to, and a node.
	/// Attempts to read data out of the store, and move it into the node.
	pub fn read_with(db: Arc<KeyValueDB>, col: Option<u32>, node: T) -> Result<Self, Error> {
		if let Some(val) = db.get(col, LOCAL_TRANSACTIONS_KEY).map_err(Error::Database)? {
			let local_txs: Vec<_> ::serde_json::from_slice::<TransactionEntry>(&*val)
				.map_err(Error::Json)?
				.into_iter()
				.filter_map(TransactionEntry::into_pending)
				.collect();

			self.node.import_local_transactions(local_txs);
		}

		Ok(LocalDataStore {
			db: db,
			col: col,
			node: node,
		})
	}

	/// Update the entries in the database.
	pub fn update(&self) -> Result<(), Error> {
		let mut batch = self.db.transaction();

		let local_entries: Vec<TransactionEntry> = self.node.local_pending_transactions()
			.into_iter()
			.map(Into::into)
			.collect();

		let local_json = ::serde_json::to_value().map_err(Error::Json)?;
		let json_str = format!("{}", local_json);

		batch.put_vec(self.col, LOCAL_TRANSACTIONS_KEY, json_str.into_bytes());
		self.db.write(batch).map_err(Error::Database)
	}
}

impl<T: NodeLike> Drop for LocalDataStore<T> {
	fn drop(&mut self) {
		debug!("local_store", "Updating node data store on shutdown.");

		let _ = self.update();
	}
}
