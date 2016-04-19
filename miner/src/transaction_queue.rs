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

// TODO [todr] - own transactions should have higher priority

//! Transaction Queue
//!
//! `TransactionQueue` keeps track of all transactions seen by the node (received from other peers) and own transactions
//! and orders them by priority. Top priority transactions are those with low nonce height (difference between
//! transaction's nonce and next nonce expected from this sender). If nonces are equal transaction's gas price is used
//! for comparison (higher gas price = higher priority).
//!
//! # Usage Example
//!
//! ```rust
//! extern crate ethcore_util as util;
//! extern crate ethcore;
//! extern crate ethminer;
//! extern crate rustc_serialize;
//!
//!	use util::crypto::KeyPair;
//! use util::hash::Address;
//! use util::numbers::{Uint, U256};
//!	use ethminer::{TransactionQueue, AccountDetails};
//!	use ethcore::transaction::*;
//!	use rustc_serialize::hex::FromHex;
//!
//! fn main() {
//!		let key = KeyPair::create().unwrap();
//!		let t1 = Transaction { action: Action::Create, value: U256::from(100), data: "3331600055".from_hex().unwrap(),
//!			gas: U256::from(100_000), gas_price: U256::one(), nonce: U256::from(10) };
//!		let t2 = Transaction { action: Action::Create, value: U256::from(100), data: "3331600055".from_hex().unwrap(),
//!			gas: U256::from(100_000), gas_price: U256::one(), nonce: U256::from(11) };
//!
//!		let st1 = t1.sign(&key.secret());
//!		let st2 = t2.sign(&key.secret());
//!		let default_nonce = |_a: &Address| AccountDetails {
//!			nonce: U256::from(10),
//!			balance: U256::from(1_000_000),
//!		};
//!
//!		let mut txq = TransactionQueue::new();
//!		txq.add(st2.clone(), &default_nonce).unwrap();
//!		txq.add(st1.clone(), &default_nonce).unwrap();
//!
//!		// Check status
//!		assert_eq!(txq.status().pending, 2);
//!		// Check top transactions
//!		let top = txq.top_transactions();
//!		assert_eq!(top.len(), 2);
//!		assert_eq!(top[0], st1);
//!		assert_eq!(top[1], st2);
//!
//!		// And when transaction is removed (but nonce haven't changed)
//!		// it will move subsequent transactions to future
//!		txq.remove_invalid(&st1.hash(), &default_nonce);
//!		assert_eq!(txq.status().pending, 0);
//!		assert_eq!(txq.status().future, 1);
//!		assert_eq!(txq.top_transactions().len(), 0);
//!	}
//! ```
//!
//!	# Maintaing valid state
//!
//!	1. Whenever transaction is imported to queue (to queue) all other transactions from this sender are revalidated in current. It means that they are moved to future and back again (height recalculation & gap filling).
//!	2. Whenever invalid transaction is removed:
//!		- When it's removed from `future` - all `future` transactions heights are recalculated and then
//!		  we check if the transactions should go to `current` (comparing state nonce)
//!		- When it's removed from `current` - all transactions from this sender (`current` & `future`) are recalculated.
//!	3. `remove_all` is used to inform the queue about client (state) nonce changes.
//!      - It removes all transactions (either from `current` or `future`) with nonce < client nonce
//!      - It moves matching `future` transactions to `current`

use std::default::Default;
use std::cmp::{Ordering};
use std::cmp;
use std::collections::{HashMap, BTreeSet};
use util::numbers::{Uint, U256};
use util::hash::{Address, H256};
use util::table::*;
use ethcore::transaction::*;
use ethcore::error::{Error, TransactionError};

#[derive(Clone, Debug)]
/// Light structure used to identify transaction and it's order
struct TransactionOrder {
	/// Primary ordering factory. Difference between transaction nonce and expected nonce in state
	/// (e.g. Tx(nonce:5), State(nonce:0) -> height: 5)
	/// High nonce_height = Low priority (processed later)
	nonce_height: U256,
	/// Gas Price of the transaction.
	/// Low gas price = Low priority (processed later)
	gas_price: U256,
	/// Hash to identify associated transaction
	hash: H256,
}


impl TransactionOrder {
	fn for_transaction(tx: &VerifiedTransaction, base_nonce: U256) -> Self {
		TransactionOrder {
			nonce_height: tx.nonce() - base_nonce,
			gas_price: tx.transaction.gas_price,
			hash: tx.hash(),
		}
	}

	fn update_height(mut self, nonce: U256, base_nonce: U256) -> Self {
		self.nonce_height = nonce - base_nonce;
		self
	}
}

impl Eq for TransactionOrder {}
impl PartialEq for TransactionOrder {
	fn eq(&self, other: &TransactionOrder) -> bool {
		self.cmp(other) == Ordering::Equal
	}
}
impl PartialOrd for TransactionOrder {
	fn partial_cmp(&self, other: &TransactionOrder) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for TransactionOrder {
	fn cmp(&self, b: &TransactionOrder) -> Ordering {
		// First check nonce_height
		if self.nonce_height != b.nonce_height {
			return self.nonce_height.cmp(&b.nonce_height);
		}

		// Then compare gas_prices
		let a_gas = self.gas_price;
		let b_gas = b.gas_price;
		if a_gas != b_gas {
			return b_gas.cmp(&a_gas);
		}

		// Compare hashes
		self.hash.cmp(&b.hash)
	}
}

/// Verified transaction (with sender)
struct VerifiedTransaction {
	transaction: SignedTransaction
}
impl VerifiedTransaction {
	fn new(transaction: SignedTransaction) -> Result<Self, Error> {
		try!(transaction.sender());
		Ok(VerifiedTransaction {
			transaction: transaction
		})
	}

	fn hash(&self) -> H256 {
		self.transaction.hash()
	}

	fn nonce(&self) -> U256 {
		self.transaction.nonce
	}

	fn sender(&self) -> Address {
		self.transaction.sender().unwrap()
	}
}

/// Holds transactions accessible by (address, nonce) and by priority
///
/// `TransactionSet` keeps number of entries below limit, but it doesn't
/// automatically happen during `insert/remove` operations.
/// You have to call `enforce_limit` to remove lowest priority transactions from set.
struct TransactionSet {
	by_priority: BTreeSet<TransactionOrder>,
	by_address: Table<Address, U256, TransactionOrder>,
	limit: usize,
}

impl TransactionSet {
	/// Inserts `TransactionOrder` to this set
	fn insert(&mut self, sender: Address, nonce: U256, order: TransactionOrder) -> Option<TransactionOrder> {
		self.by_priority.insert(order.clone());
		let r = self.by_address.insert(sender, nonce, order);
		// If transaction was replaced remove it from priority queue
		if let Some(ref order) = r {
			self.by_priority.remove(order);
		}
		r
	}

	/// Remove low priority transactions if there is more then specified by given `limit`.
	///
	/// It drops transactions from this set but also removes associated `VerifiedTransaction`.
	/// Returns addresses and highes nonces of transactions removed because of limit.
	fn enforce_limit(&mut self, by_hash: &mut HashMap<H256, VerifiedTransaction>) -> Option<HashMap<Address, U256>> {
		let len = self.by_priority.len();
		if len <= self.limit {
			return None;
		}

		let to_drop : Vec<(Address, U256)> = {
			self.by_priority
				.iter()
				.skip(self.limit)
				.map(|order| by_hash.get(&order.hash)
					.expect("All transactions in `self.by_priority` and `self.by_address` are kept in sync with `by_hash`."))
				.map(|tx| (tx.sender(), tx.nonce()))
				.collect()
		};

		Some(to_drop.into_iter()
			.fold(HashMap::new(), |mut removed, (sender, nonce)| {
				let order = self.drop(&sender, &nonce)
					.expect("Transaction has just been found in `by_priority`; so it is in `by_address` also.");

				by_hash.remove(&order.hash)
					.expect("Hash found in `by_priorty` matches the one dropped; so it is included in `by_hash`");

				let max = removed.get(&sender).map(|val| cmp::max(*val, nonce)).unwrap_or(nonce);
				removed.insert(sender, max);
				removed
			}))
	}

	/// Drop transaction from this set (remove from `by_priority` and `by_address`)
	fn drop(&mut self, sender: &Address, nonce: &U256) -> Option<TransactionOrder> {
		if let Some(tx_order) = self.by_address.remove(sender, nonce) {
			self.by_priority.remove(&tx_order);
			return Some(tx_order);
		}
		None
	}

	/// Drop all transactions.
	fn clear(&mut self) {
		self.by_priority.clear();
		self.by_address.clear();
	}

	/// Sets new limit for number of transactions in this `TransactionSet`.
	/// Note the limit is not applied (no transactions are removed) by calling this method.
	fn set_limit(&mut self, limit: usize) {
		self.limit = limit;
	}
}

#[derive(Debug)]
/// Current status of the queue
pub struct TransactionQueueStatus {
	/// Number of pending transactions (ready to go to block)
	pub pending: usize,
	/// Number of future transactions (waiting for transactions with lower nonces first)
	pub future: usize,
}

#[derive(Debug, PartialEq)]
/// Represents the result of importing transaction.
pub enum TransactionImportResult {
	/// Transaction was imported to current queue.
	Current,
	/// Transaction was imported to future queue.
	Future
}

/// Details of account
pub struct AccountDetails {
	/// Most recent account nonce
	pub nonce: U256,
	/// Current account balance
	pub balance: U256,
}


/// Transactions with `gas > (gas_limit + gas_limit * Factor(in percents))` are not imported to the queue.
const GAS_LIMIT_HYSTERESIS: usize = 10; // %

/// `TransactionQueue` implementation
pub struct TransactionQueue {
	/// Gas Price threshold for transactions that can be imported to this queue (defaults to 0)
	minimal_gas_price: U256,
	/// Current gas limit (block gas limit * factor). Transactions above the limit will not be accepted (default to !0)
	gas_limit: U256,
	/// Priority queue for transactions that can go to block
	current: TransactionSet,
	/// Priority queue for transactions that has been received but are not yet valid to go to block
	future: TransactionSet,
	/// All transactions managed by queue indexed by hash
	by_hash: HashMap<H256, VerifiedTransaction>,
	/// Last nonce of transaction in current (to quickly check next expected transaction)
	last_nonces: HashMap<Address, U256>,
}

impl Default for TransactionQueue {
	fn default() -> Self {
		TransactionQueue::new()
	}
}

impl TransactionQueue {
	/// Creates new instance of this Queue
	pub fn new() -> Self {
		Self::with_limit(1024)
	}

	/// Create new instance of this Queue with specified limits
	pub fn with_limit(limit: usize) -> Self {
		let current = TransactionSet {
			by_priority: BTreeSet::new(),
			by_address: Table::new(),
			limit: limit,
		};

		let future = TransactionSet {
			by_priority: BTreeSet::new(),
			by_address: Table::new(),
			limit: limit,
		};

		TransactionQueue {
			minimal_gas_price: U256::zero(),
			gas_limit: !U256::zero(),
			current: current,
			future: future,
			by_hash: HashMap::new(),
			last_nonces: HashMap::new(),
		}
	}

	/// Set the new limit for `current` and `future` queue.
	pub fn set_limit(&mut self, limit: usize) {
		self.current.set_limit(limit);
		self.future.set_limit(limit);
		// And ensure the limits
		self.current.enforce_limit(&mut self.by_hash);
		self.future.enforce_limit(&mut self.by_hash);
	}

	/// Returns current limit of transactions in the queue.
	pub fn limit(&self) -> usize {
		self.current.limit
	}

	/// Get the minimal gas price.
	pub fn minimal_gas_price(&self) -> &U256 {
		&self.minimal_gas_price
	}

	/// Sets new gas price threshold for incoming transactions.
	/// Any transaction already imported to the queue is not affected.
	pub fn set_minimal_gas_price(&mut self, min_gas_price: U256) {
		self.minimal_gas_price = min_gas_price;
	}

	/// Sets new gas limit. Transactions with gas slightly (`GAS_LIMIT_HYSTERESIS`) above the limit won't be imported.
	/// Any transaction already imported to the queue is not affected.
	pub fn set_gas_limit(&mut self, gas_limit: U256) {
		let extra = gas_limit / U256::from(GAS_LIMIT_HYSTERESIS);

		self.gas_limit = match gas_limit.overflowing_add(extra) {
			(_, true) => !U256::zero(),
			(val, false) => val,
		};
	}

	/// Returns current status for this queue
	pub fn status(&self) -> TransactionQueueStatus {
		TransactionQueueStatus {
			pending: self.current.by_priority.len(),
			future: self.future.by_priority.len(),
		}
	}

	/// Adds all signed transactions to queue to be verified and imported
	pub fn add_all<T>(&mut self, txs: Vec<SignedTransaction>, fetch_account: T) -> Vec<Result<TransactionImportResult, Error>>
		where T: Fn(&Address) -> AccountDetails {

		txs.into_iter()
			.map(|tx| self.add(tx, &fetch_account))
			.collect()
	}

	/// Add signed transaction to queue to be verified and imported
	pub fn add<T>(&mut self, tx: SignedTransaction, fetch_account: &T) -> Result<TransactionImportResult, Error>
		where T: Fn(&Address) -> AccountDetails {

		trace!(target: "miner", "Importing: {:?}", tx.hash());

		if tx.gas_price < self.minimal_gas_price {
			trace!(target: "miner",
				"Dropping transaction below minimal gas price threshold: {:?} (gp: {} < {})",
				tx.hash(), tx.gas_price, self.minimal_gas_price
			);

			return Err(Error::Transaction(TransactionError::InsufficientGasPrice {
				minimal: self.minimal_gas_price,
				got: tx.gas_price,
			}));
		}

		if tx.gas > self.gas_limit {
			trace!(target: "miner",
				"Dropping transaction above gas limit: {:?} ({} > {})",
				tx.hash(), tx.gas, self.gas_limit
			);

			return Err(Error::Transaction(TransactionError::GasLimitExceeded {
				limit: self.gas_limit,
				got: tx.gas,
			}));
		}

		let vtx = try!(VerifiedTransaction::new(tx));
		let client_account = fetch_account(&vtx.sender());

		let cost = vtx.transaction.value + vtx.transaction.gas_price * vtx.transaction.gas;
		if client_account.balance < cost {
			trace!(target: "miner", "Dropping transaction without sufficient balance: {:?} ({} < {})",
				vtx.hash(), client_account.balance, cost);
			return Err(Error::Transaction(TransactionError::InsufficientBalance {
				cost: cost,
				balance: client_account.balance
			}));
		}

		self.import_tx(vtx, client_account.nonce).map_err(Error::Transaction)
	}

	/// Removes all transactions from particular sender up to (excluding) given client (state) nonce.
	/// Client (State) Nonce = next valid nonce for this sender.
	pub fn remove_all(&mut self, sender: Address, client_nonce: U256) {
		// We will either move transaction to future or remove it completely
		// so there will be no transactions from this sender in current
		self.last_nonces.remove(&sender);
		// First update height of transactions in future to avoid collisions
		self.update_future(&sender, client_nonce);
		// This should move all current transactions to future and remove old transactions
		self.move_all_to_future(&sender, client_nonce);
		// And now lets check if there is some batch of transactions in future
		// that should be placed in current. It should also update last_nonces.
		self.move_matching_future_to_current(sender, client_nonce, client_nonce);
	}

	/// Removes invalid transaction identified by hash from queue.
	/// Assumption is that this transaction nonce is not related to client nonce,
	/// so transactions left in queue are processed according to client nonce.
	///
	/// If gap is introduced marks subsequent transactions as future
	pub fn remove_invalid<T>(&mut self, transaction_hash: &H256, fetch_account: &T)
		where T: Fn(&Address) -> AccountDetails {
		let transaction = self.by_hash.remove(transaction_hash);
		if transaction.is_none() {
			// We don't know this transaction
			return;
		}

		let transaction = transaction.unwrap();
		let sender = transaction.sender();
		let nonce = transaction.nonce();
		let current_nonce = fetch_account(&sender).nonce;

		// Remove from future
		let order = self.future.drop(&sender, &nonce);
		if order.is_some() {
			self.update_future(&sender, current_nonce);
			// And now lets check if there is some chain of transactions in future
			// that should be placed in current
			self.move_matching_future_to_current(sender, current_nonce, current_nonce);
			return;
		}

		// Remove from current
		let order = self.current.drop(&sender, &nonce);
		if order.is_some() {
			// We will either move transaction to future or remove it completely
			// so there will be no transactions from this sender in current
			self.last_nonces.remove(&sender);
			// First update height of transactions in future to avoid collisions
			self.update_future(&sender, current_nonce);
			// This should move all current transactions to future and remove old transactions
			self.move_all_to_future(&sender, current_nonce);
			// And now lets check if there is some chain of transactions in future
			// that should be placed in current. It should also update last_nonces.
			self.move_matching_future_to_current(sender, current_nonce, current_nonce);
			return;
		}
	}

	/// Update height of all transactions in future transactions set.
	fn update_future(&mut self, sender: &Address, current_nonce: U256) {
		// We need to drain all transactions for current sender from future and reinsert them with updated height
		let all_nonces_from_sender = match self.future.by_address.row(&sender) {
			Some(row_map) => row_map.keys().cloned().collect::<Vec<U256>>(),
			None => vec![],
		};
		for k in all_nonces_from_sender {
			let order = self.future.drop(&sender, &k).unwrap();
			if k >= current_nonce {
				self.future.insert(*sender, k, order.update_height(k, current_nonce));
			} else {
				trace!(target: "miner", "Removing old transaction: {:?} (nonce: {} < {})", order.hash, k, current_nonce);
				// Remove the transaction completely
				self.by_hash.remove(&order.hash);
			}
		}
	}

	/// Drop all transactions from given sender from `current`.
	/// Either moves them to `future` or removes them from queue completely.
	fn move_all_to_future(&mut self, sender: &Address, current_nonce: U256) {
		let all_nonces_from_sender = match self.current.by_address.row(&sender) {
			Some(row_map) => row_map.keys().cloned().collect::<Vec<U256>>(),
			None => vec![],
		};

		for k in all_nonces_from_sender {
			// Goes to future or is removed
			let order = self.current.drop(&sender, &k).unwrap();
			if k >= current_nonce {
				self.future.insert(*sender, k, order.update_height(k, current_nonce));
			} else {
				trace!(target: "miner", "Removing old transaction: {:?} (nonce: {} < {})", order.hash, k, current_nonce);
				self.by_hash.remove(&order.hash);
			}
		}
		self.future.enforce_limit(&mut self.by_hash);
	}

	/// Returns top transactions from the queue ordered by priority.
	pub fn top_transactions(&self) -> Vec<SignedTransaction> {
		self.current.by_priority
			.iter()
			.map(|t| self.by_hash.get(&t.hash).expect("All transactions in `current` and `future` are always included in `by_hash`"))
			.map(|t| t.transaction.clone())
			.collect()
	}

	/// Returns hashes of all transactions from current, ordered by priority.
	pub fn pending_hashes(&self) -> Vec<H256> {
		self.current.by_priority
			.iter()
			.map(|t| t.hash)
			.collect()
	}

	/// Finds transaction in the queue by hash (if any)
	pub fn find(&self, hash: &H256) -> Option<SignedTransaction> {
		match self.by_hash.get(hash) { Some(transaction_ref) => Some(transaction_ref.transaction.clone()), None => None }
	}

	/// Removes all elements (in any state) from the queue
	pub fn clear(&mut self) {
		self.current.clear();
		self.future.clear();
		self.by_hash.clear();
		self.last_nonces.clear();
	}

	/// Returns highest transaction nonce for given address.
	pub fn last_nonce(&self, address: &Address) -> Option<U256> {
		self.last_nonces.get(address).cloned()
	}

	/// Checks if there are any transactions in `future` that should actually be promoted to `current`
	/// (because nonce matches).
	fn move_matching_future_to_current(&mut self, address: Address, mut current_nonce: U256, first_nonce: U256) {
		{
			let by_nonce = self.future.by_address.row_mut(&address);
			if let None = by_nonce {
				return;
			}
			let mut by_nonce = by_nonce.unwrap();
			while let Some(order) = by_nonce.remove(&current_nonce) {
				// remove also from priority and hash
				self.future.by_priority.remove(&order);
				// Put to current
				let order = order.update_height(current_nonce, first_nonce);
				self.current.insert(address, current_nonce, order);
				current_nonce = current_nonce + U256::one();
			}
		}
		self.future.by_address.clear_if_empty(&address);
		// Update last inserted nonce
		self.last_nonces.insert(address, current_nonce - U256::one());
	}

	/// Adds VerifiedTransaction to this queue.
	///
	/// Determines if it should be placed in current or future. When transaction is
	/// imported to `current` also checks if there are any `future` transactions that should be promoted because of
	/// this.
	///
	/// It ignores transactions that has already been imported (same `hash`) and replaces the transaction
	/// iff `(address, nonce)` is the same but `gas_price` is higher.
	///
	/// Returns `true` when transaction was imported successfuly
	fn import_tx(&mut self, tx: VerifiedTransaction, state_nonce: U256) -> Result<TransactionImportResult, TransactionError> {

		if self.by_hash.get(&tx.hash()).is_some() {
			// Transaction is already imported.
			trace!(target: "miner", "Dropping already imported transaction: {:?}", tx.hash());
			return Err(TransactionError::AlreadyImported);
		}


		let address = tx.sender();
		let nonce = tx.nonce();

		let next_nonce = self.last_nonces
			.get(&address)
			.cloned()
			.map_or(state_nonce, |n| n + U256::one());

		// Check height
		if nonce > next_nonce {
			// We have a gap - put to future
			try!(check_too_cheap(Self::replace_transaction(tx, next_nonce, &mut self.future, &mut self.by_hash)));
			try!(check_if_removed(&address, &nonce, self.future.enforce_limit(&mut self.by_hash)));
			return Ok(TransactionImportResult::Future);
		} else if nonce < state_nonce {
			// Droping transaction
			trace!(target: "miner", "Dropping old transaction: {:?} (nonce: {} < {})", tx.hash(), nonce, next_nonce);
			return Err(TransactionError::Old);
		}

		try!(check_too_cheap(Self::replace_transaction(tx, state_nonce, &mut self.current, &mut self.by_hash)));
		// Keep track of highest nonce stored in current
		self.last_nonces.insert(address, nonce);
		// Update nonces of transactions in future
		self.update_future(&address, state_nonce);
		// Maybe there are some more items waiting in future?
		self.move_matching_future_to_current(address, nonce + U256::one(), state_nonce);
		// There might be exactly the same transaction waiting in future
		// same (sender, nonce), but above function would not move it.
		if let Some(order) = self.future.drop(&address, &nonce) {
			// Let's insert that transaction to current (if it has higher gas_price)
			let future_tx = self.by_hash.remove(&order.hash).unwrap();
			try!(check_too_cheap(Self::replace_transaction(future_tx, state_nonce, &mut self.current, &mut self.by_hash)));
		}

		// Also enforce the limit
		let removed = self.current.enforce_limit(&mut self.by_hash);
		// If some transaction were removed because of limit we need to update last_nonces also.
		self.update_last_nonces(&removed);
		// Trigger error if we were removed.
		try!(check_if_removed(&address, &nonce, removed));

		trace!(target: "miner", "status: {:?}", self.status());
		Ok(TransactionImportResult::Current)
	}

	/// Updates
	fn update_last_nonces(&mut self, removed_max_nonces: &Option<HashMap<Address, U256>>) {
		if let Some(ref max_nonces) = *removed_max_nonces {
			for (sender, nonce) in max_nonces.iter() {
				if *nonce == U256::zero() {
					self.last_nonces.remove(sender);
				} else {
					self.last_nonces.insert(*sender, *nonce - U256::one());
				}
			}
		}
	}

	/// Replaces transaction in given set (could be `future` or `current`).
	///
	/// If there is already transaction with same `(sender, nonce)` it will be replaced iff `gas_price` is higher.
	/// One of the transactions is dropped from set and also removed from queue entirely (from `by_hash`).
	///
	/// Returns `true` if transaction actually got to the queue (`false` if there was already a transaction with higher
	/// gas_price)
	fn replace_transaction(tx: VerifiedTransaction, base_nonce: U256, set: &mut TransactionSet, by_hash: &mut HashMap<H256, VerifiedTransaction>) -> bool {
		let order = TransactionOrder::for_transaction(&tx, base_nonce);
		let hash = tx.hash();
		let address = tx.sender();
		let nonce = tx.nonce();

		by_hash.insert(hash, tx);

		if let Some(old) = set.insert(address, nonce, order.clone()) {
			// There was already transaction in queue. Let's check which one should stay
			let old_fee = old.gas_price;
			let new_fee = order.gas_price;
			if old_fee.cmp(&new_fee) == Ordering::Greater {
				// Put back old transaction since it has greater priority (higher gas_price)
				set.insert(address, nonce, old);
				// and remove new one
				by_hash.remove(&hash);
				false
			} else {
				// Make sure we remove old transaction entirely
				by_hash.remove(&old.hash);
				true
			}
		} else {
			true
		}
	}
}

fn check_too_cheap(is_in: bool) -> Result<(), TransactionError> {
	if !is_in {
		Err(TransactionError::TooCheapToReplace)
	} else {
		Ok(())
	}
}

fn check_if_removed(sender: &Address, nonce: &U256, dropped: Option<HashMap<Address, U256>>) -> Result<(), TransactionError> {
	match dropped {
		Some(ref dropped) => match dropped.get(sender) {
			Some(max) if nonce <= max => {
				Err(TransactionError::LimitReached)
			},
			_ => Ok(()),
		},
		_ => Ok(()),
	}
}


#[cfg(test)]
mod test {
	extern crate rustc_serialize;
	use util::table::*;
	use util::*;
	use ethcore::transaction::*;
	use ethcore::error::{Error, TransactionError};
	use super::*;
	use super::{TransactionSet, TransactionOrder, VerifiedTransaction};

	fn unwrap_tx_err(err: Result<TransactionImportResult, Error>) -> TransactionError {
		match err.unwrap_err() {
			Error::Transaction(e) => e,
			_ => panic!("Expected transaction error!"),
		}
	}

	fn new_unsigned_tx(nonce: U256) -> Transaction {
		Transaction {
			action: Action::Create,
			value: U256::from(100),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::one(),
			nonce: nonce
		}
	}

	fn new_tx() -> SignedTransaction {
		let keypair = KeyPair::create().unwrap();
		new_unsigned_tx(U256::from(123)).sign(&keypair.secret())
	}


	fn default_nonce_val() -> U256 {
		U256::from(123)
	}

	fn default_nonce(_address: &Address) -> AccountDetails {
		AccountDetails {
			nonce: default_nonce_val(),
			balance: !U256::zero()
		}
	}

	/// Returns two transactions with identical (sender, nonce) but different hashes
	fn new_similar_txs() -> (SignedTransaction, SignedTransaction) {
		let keypair = KeyPair::create().unwrap();
		let secret = &keypair.secret();
		let nonce = U256::from(123);
		let tx = new_unsigned_tx(nonce);
		let mut tx2 = new_unsigned_tx(nonce);
		tx2.gas_price = U256::from(2);

		(tx.sign(secret), tx2.sign(secret))
	}

	fn new_txs(second_nonce: U256) -> (SignedTransaction, SignedTransaction) {
		new_txs_with_gas_price_diff(second_nonce, U256::zero())
	}

	fn new_txs_with_gas_price_diff(second_nonce: U256, gas_price: U256) -> (SignedTransaction, SignedTransaction) {
		let keypair = KeyPair::create().unwrap();
		let secret = &keypair.secret();
		let nonce = U256::from(123);
		let tx = new_unsigned_tx(nonce);
		let mut tx2 = new_unsigned_tx(nonce + second_nonce);
		tx2.gas_price = tx2.gas_price + gas_price;

		(tx.sign(secret), tx2.sign(secret))
	}

	#[test]
	fn should_create_transaction_set() {
		// given
		let mut set = TransactionSet {
			by_priority: BTreeSet::new(),
			by_address: Table::new(),
			limit: 1
		};
		let (tx1, tx2) = new_txs(U256::from(1));
		let tx1 = VerifiedTransaction::new(tx1).unwrap();
		let tx2 = VerifiedTransaction::new(tx2).unwrap();
		let mut by_hash = {
			let mut x = HashMap::new();
			let tx1 = VerifiedTransaction::new(tx1.transaction.clone()).unwrap();
			let tx2 = VerifiedTransaction::new(tx2.transaction.clone()).unwrap();
			x.insert(tx1.hash(), tx1);
			x.insert(tx2.hash(), tx2);
			x
		};
		// Insert both transactions
		let order1 = TransactionOrder::for_transaction(&tx1, U256::zero());
		set.insert(tx1.sender(), tx1.nonce(), order1.clone());
		let order2 = TransactionOrder::for_transaction(&tx2, U256::zero());
		set.insert(tx2.sender(), tx2.nonce(), order2.clone());
		assert_eq!(set.by_priority.len(), 2);
		assert_eq!(set.by_address.len(), 2);

		// when
		set.enforce_limit(&mut by_hash);

		// then
		assert_eq!(by_hash.len(), 1);
		assert_eq!(set.by_priority.len(), 1);
		assert_eq!(set.by_address.len(), 1);
		assert_eq!(set.by_priority.iter().next().unwrap().clone(), order1);
		set.clear();
		assert_eq!(set.by_priority.len(), 0);
		assert_eq!(set.by_address.len(), 0);
	}

	#[test]
	fn should_replace_transaction_in_set() {
		let mut set = TransactionSet {
			by_priority: BTreeSet::new(),
			by_address: Table::new(),
			limit: 1
		};
		// Create two transactions with same nonce
		// (same hash)
		let (tx1, tx2) = new_txs(U256::from(0));
		let tx1 = VerifiedTransaction::new(tx1).unwrap();
		let tx2 = VerifiedTransaction::new(tx2).unwrap();
		let by_hash = {
			let mut x = HashMap::new();
			let tx1 = VerifiedTransaction::new(tx1.transaction.clone()).unwrap();
			let tx2 = VerifiedTransaction::new(tx2.transaction.clone()).unwrap();
			x.insert(tx1.hash(), tx1);
			x.insert(tx2.hash(), tx2);
			x
		};
		// Insert both transactions
		let order1 = TransactionOrder::for_transaction(&tx1, U256::zero());
		set.insert(tx1.sender(), tx1.nonce(), order1.clone());
		assert_eq!(set.by_priority.len(), 1);
		assert_eq!(set.by_address.len(), 1);
		// Two different orders (imagine nonce changed in the meantime)
		let order2 = TransactionOrder::for_transaction(&tx2, U256::one());
		set.insert(tx2.sender(), tx2.nonce(), order2.clone());
		assert_eq!(set.by_priority.len(), 1);
		assert_eq!(set.by_address.len(), 1);

		// then
		assert_eq!(by_hash.len(), 1);
		assert_eq!(set.by_priority.len(), 1);
		assert_eq!(set.by_address.len(), 1);
		assert_eq!(set.by_priority.iter().next().unwrap().clone(), order2);
	}

	#[test]
	fn should_handle_same_transaction_imported_twice_with_different_state_nonces() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx, tx2) = new_similar_txs();
		let prev_nonce = |a: &Address| AccountDetails{ nonce: default_nonce(a).nonce - U256::one(), balance:
			!U256::zero() };

		// First insert one transaction to future
		let res = txq.add(tx, &prev_nonce);
		assert_eq!(res.unwrap(), TransactionImportResult::Future);
		assert_eq!(txq.status().future, 1);

		// now import second transaction to current
		let res = txq.add(tx2.clone(), &default_nonce);

		// and then there should be only one transaction in current (the one with higher gas_price)
		assert_eq!(unwrap_tx_err(res), TransactionError::TooCheapToReplace);
		assert_eq!(txq.status().pending, 1);
		assert_eq!(txq.status().future, 0);
		assert_eq!(txq.current.by_priority.len(), 1);
		assert_eq!(txq.current.by_address.len(), 1);
		assert_eq!(txq.top_transactions()[0], tx2);
	}


	#[test]
	fn should_import_tx() {
		// given
		let mut txq = TransactionQueue::new();
		let tx = new_tx();

		// when
		let res = txq.add(tx, &default_nonce);

		// then
		assert_eq!(res.unwrap(), TransactionImportResult::Current);
		let stats = txq.status();
		assert_eq!(stats.pending, 1);
	}

	#[test]
	fn gas_limit_should_never_overflow() {
		// given
		let mut txq = TransactionQueue::new();
		txq.set_gas_limit(U256::zero());
		assert_eq!(txq.gas_limit, U256::zero());

		// when
		txq.set_gas_limit(!U256::zero());

		// then
		assert_eq!(txq.gas_limit, !U256::zero());
	}

	#[test]
	fn should_not_import_transaction_above_gas_limit() {
		// given
		let mut txq = TransactionQueue::new();
		let tx = new_tx();
		let gas = tx.gas;
		let limit = gas / U256::from(2);
		txq.set_gas_limit(limit);

		// when
		let res = txq.add(tx, &default_nonce);

		// then
		assert_eq!(unwrap_tx_err(res), TransactionError::GasLimitExceeded {
			limit: U256::from(55_000), // Should be 110% of set_gas_limit
			got: gas,
		});
		let stats = txq.status();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
	}


	#[test]
	fn should_drop_transactions_from_senders_without_balance() {
		// given
		let mut txq = TransactionQueue::new();
		let tx = new_tx();
		let account = |a: &Address| AccountDetails {
			nonce: default_nonce(a).nonce,
			balance: U256::one()
		};

		// when
		let res = txq.add(tx, &account);

		// then
		assert_eq!(unwrap_tx_err(res), TransactionError::InsufficientBalance {
			balance: U256::from(1),
			cost: U256::from(100_100),
		});
		let stats = txq.status();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_not_import_transaction_below_min_gas_price_threshold() {
		// given
		let mut txq = TransactionQueue::new();
		let tx = new_tx();
		txq.set_minimal_gas_price(tx.gas_price + U256::one());

		// when
		let res = txq.add(tx, &default_nonce);

		// then
		assert_eq!(unwrap_tx_err(res), TransactionError::InsufficientGasPrice {
			minimal: U256::from(2),
			got: U256::from(1),
		});
		let stats = txq.status();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_reject_incorectly_signed_transaction() {
		// given
		let mut txq = TransactionQueue::new();
		let tx = new_unsigned_tx(U256::from(123));
		let stx = {
			let mut s = RlpStream::new_list(9);
			s.append(&tx.nonce);
			s.append(&tx.gas_price);
			s.append(&tx.gas);
			s.append_empty_data(); // action=create
			s.append(&tx.value);
			s.append(&tx.data);
			s.append(&0u64); // v
			s.append(&U256::zero()); // r
			s.append(&U256::zero()); // s
			decode(s.as_raw())
		};
		// when
		let res = txq.add(stx, &default_nonce);

		// then
		assert!(res.is_err());
	}

	#[test]
	fn should_import_txs_from_same_sender() {
		// given
		let mut txq = TransactionQueue::new();

		let (tx, tx2) = new_txs(U256::from(1));

		// when
		txq.add(tx.clone(), &default_nonce).unwrap();
		txq.add(tx2.clone(), &default_nonce).unwrap();

		// then
		let top = txq.top_transactions();
		assert_eq!(top[0], tx);
		assert_eq!(top[1], tx2);
		assert_eq!(top.len(), 2);
	}

	#[test]
	fn should_return_pending_hashes() {
			// given
		let mut txq = TransactionQueue::new();

		let (tx, tx2) = new_txs(U256::from(1));

		// when
		txq.add(tx.clone(), &default_nonce).unwrap();
		txq.add(tx2.clone(), &default_nonce).unwrap();

		// then
		let top = txq.pending_hashes();
		assert_eq!(top[0], tx.hash());
		assert_eq!(top[1], tx2.hash());
		assert_eq!(top.len(), 2);
	}

	#[test]
	fn should_put_transaction_to_futures_if_gap_detected() {
		// given
		let mut txq = TransactionQueue::new();

		let (tx, tx2) = new_txs(U256::from(2));

		// when
		let res1 = txq.add(tx.clone(), &default_nonce).unwrap();
		let res2 = txq.add(tx2.clone(), &default_nonce).unwrap();

		// then
		assert_eq!(res1, TransactionImportResult::Current);
		assert_eq!(res2, TransactionImportResult::Future);
		let stats = txq.status();
		assert_eq!(stats.pending, 1);
		assert_eq!(stats.future, 1);
		let top = txq.top_transactions();
		assert_eq!(top.len(), 1);
		assert_eq!(top[0], tx);
	}

	#[test]
	fn should_correctly_update_futures_when_removing() {
		// given
		let prev_nonce = |a: &Address| AccountDetails{ nonce: default_nonce(a).nonce - U256::one(), balance:
			!U256::zero() };
		let next2_nonce = default_nonce_val() + U256::from(3);

		let mut txq = TransactionQueue::new();

		let (tx, tx2) = new_txs(U256::from(1));
		txq.add(tx.clone(), &prev_nonce).unwrap();
		txq.add(tx2.clone(), &prev_nonce).unwrap();
		assert_eq!(txq.status().future, 2);

		// when
		txq.remove_all(tx.sender().unwrap(), next2_nonce);
		// should remove both transactions since they are not valid

		// then
		assert_eq!(txq.status().pending, 0);
		assert_eq!(txq.status().future, 0);
	}

	#[test]
	fn should_move_transactions_if_gap_filled() {
		// given
		let mut txq = TransactionQueue::new();
		let kp = KeyPair::create().unwrap();
		let secret = kp.secret();
		let tx = new_unsigned_tx(U256::from(123)).sign(&secret);
		let tx1 = new_unsigned_tx(U256::from(124)).sign(&secret);
		let tx2 = new_unsigned_tx(U256::from(125)).sign(&secret);

		txq.add(tx, &default_nonce).unwrap();
		assert_eq!(txq.status().pending, 1);
		txq.add(tx2, &default_nonce).unwrap();
		assert_eq!(txq.status().future, 1);

		// when
		txq.add(tx1, &default_nonce).unwrap();

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 3);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_remove_transaction() {
		// given
		let mut txq2 = TransactionQueue::new();
		let (tx, tx2) = new_txs(U256::from(3));
		txq2.add(tx.clone(), &default_nonce).unwrap();
		txq2.add(tx2.clone(), &default_nonce).unwrap();
		assert_eq!(txq2.status().pending, 1);
		assert_eq!(txq2.status().future, 1);

		// when
		txq2.remove_all(tx.sender().unwrap(), tx.nonce + U256::one());
		txq2.remove_all(tx2.sender().unwrap(), tx2.nonce + U256::one());


		// then
		let stats = txq2.status();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_move_transactions_to_future_if_gap_introduced() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx, tx2) = new_txs(U256::from(1));
		let tx3 = new_tx();
		txq.add(tx2.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().future, 1);
		txq.add(tx3.clone(), &default_nonce).unwrap();
		txq.add(tx.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().pending, 3);

		// when
		txq.remove_invalid(&tx.hash(), &default_nonce);

		// then
		let stats = txq.status();
		assert_eq!(stats.future, 1);
		assert_eq!(stats.pending, 1);
	}

	#[test]
	fn should_clear_queue() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx, tx2) = new_txs(U256::one());

		// add
		txq.add(tx2.clone(), &default_nonce).unwrap();
		txq.add(tx.clone(), &default_nonce).unwrap();
		let stats = txq.status();
		assert_eq!(stats.pending, 2);

		// when
		txq.clear();

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 0);
	}

	#[test]
	fn should_drop_old_transactions_when_hitting_the_limit() {
		// given
		let mut txq = TransactionQueue::with_limit(1);
		let (tx, tx2) = new_txs(U256::one());
		let sender = tx.sender().unwrap();
		let nonce = tx.nonce;
		txq.add(tx.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().pending, 1);

		// when
		let res = txq.add(tx2.clone(), &default_nonce);

		// then
		let t = txq.top_transactions();
		assert_eq!(unwrap_tx_err(res), TransactionError::LimitReached);
		assert_eq!(txq.status().pending, 1);
		assert_eq!(t.len(), 1);
		assert_eq!(t[0], tx);
		assert_eq!(txq.last_nonce(&sender), Some(nonce));
	}

	#[test]
	fn should_return_correct_nonces_when_dropped_because_of_limit() {
		// given
		let mut txq = TransactionQueue::with_limit(2);
		let tx = new_tx();
		let (tx1, tx2) = new_txs(U256::one());
		let sender = tx1.sender().unwrap();
		let nonce = tx1.nonce;
		txq.add(tx1.clone(), &default_nonce).unwrap();
		txq.add(tx2.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().pending, 2);
		assert_eq!(txq.last_nonce(&sender), Some(nonce + U256::one()));

		// when
		let res = txq.add(tx.clone(), &default_nonce);

		// then
		assert_eq!(res.unwrap(), TransactionImportResult::Current);
		assert_eq!(txq.status().pending, 2);
		assert_eq!(txq.last_nonce(&sender), Some(nonce));
	}

	#[test]
	fn should_limit_future_transactions() {
		let mut txq = TransactionQueue::with_limit(1);
		txq.current.set_limit(10);
		let (tx1, tx2) = new_txs_with_gas_price_diff(U256::from(4), U256::from(1));
		let (tx3, tx4) = new_txs_with_gas_price_diff(U256::from(4), U256::from(2));
		txq.add(tx1.clone(), &default_nonce).unwrap();
		txq.add(tx3.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().pending, 2);

		// when
		txq.add(tx2.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().future, 1);
		txq.add(tx4.clone(), &default_nonce).unwrap();

		// then
		assert_eq!(txq.status().future, 1);
	}

	#[test]
	fn should_drop_transactions_with_old_nonces() {
		let mut txq = TransactionQueue::new();
		let tx = new_tx();
		let last_nonce = tx.nonce + U256::one();
		let fetch_last_nonce = |_a: &Address| AccountDetails{ nonce: last_nonce, balance: !U256::zero() };

		// when
		let res = txq.add(tx, &fetch_last_nonce);

		// then
		assert_eq!(unwrap_tx_err(res), TransactionError::Old);
		let stats = txq.status();
		assert_eq!(stats.pending, 0);
		assert_eq!(stats.future, 0);
	}

	#[test]
	fn should_not_insert_same_transaction_twice() {
		// given
		let nonce = |a: &Address| AccountDetails { nonce: default_nonce(a).nonce + U256::one(),
			balance: !U256::zero() };
		let mut txq = TransactionQueue::new();
		let (_tx1, tx2) = new_txs(U256::from(1));
		txq.add(tx2.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().future, 1);
		assert_eq!(txq.status().pending, 0);

		// when
		let res = txq.add(tx2.clone(), &nonce);

		// then
		assert_eq!(unwrap_tx_err(res), TransactionError::AlreadyImported);
		let stats = txq.status();
		assert_eq!(stats.future, 1);
		assert_eq!(stats.pending, 0);
	}

	#[test]
	fn should_accept_same_transaction_twice_if_removed() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx1, tx2) = new_txs(U256::from(1));
		txq.add(tx1.clone(), &default_nonce).unwrap();
		txq.add(tx2.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().pending, 2);

		// when
		txq.remove_invalid(&tx1.hash(), &default_nonce);
		assert_eq!(txq.status().pending, 0);
		assert_eq!(txq.status().future, 1);
		txq.add(tx1.clone(), &default_nonce).unwrap();

		// then
		let stats = txq.status();
		assert_eq!(stats.future, 0);
		assert_eq!(stats.pending, 2);
	}

	#[test]
	fn should_not_move_to_future_if_state_nonce_is_higher() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx, tx2) = new_txs(U256::from(1));
		let tx3 = new_tx();
		txq.add(tx2.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().future, 1);
		txq.add(tx3.clone(), &default_nonce).unwrap();
		txq.add(tx.clone(), &default_nonce).unwrap();
		assert_eq!(txq.status().pending, 3);

		// when
		let sender = tx.sender().unwrap();
		txq.remove_all(sender, default_nonce_val() + U256::one());

		// then
		let stats = txq.status();
		assert_eq!(stats.future, 0);
		assert_eq!(stats.pending, 2);
	}

	#[test]
	fn should_replace_same_transaction_when_has_higher_fee() {
		init_log();
		// given
		let mut txq = TransactionQueue::new();
		let keypair = KeyPair::create().unwrap();
		let tx = new_unsigned_tx(U256::from(123)).sign(&keypair.secret());
		let tx2 = {
			let mut tx2 = tx.deref().clone();
			tx2.gas_price = U256::from(200);
			tx2.sign(&keypair.secret())
		};

		// when
		txq.add(tx, &default_nonce).unwrap();
		txq.add(tx2, &default_nonce).unwrap();

		// then
		let stats = txq.status();
		assert_eq!(stats.pending, 1);
		assert_eq!(stats.future, 0);
		assert_eq!(txq.top_transactions()[0].gas_price, U256::from(200));
	}

	#[test]
	fn should_replace_same_transaction_when_importing_to_futures() {
		// given
		let mut txq = TransactionQueue::new();
		let keypair = KeyPair::create().unwrap();
		let tx0 = new_unsigned_tx(U256::from(123)).sign(&keypair.secret());
		let tx1 = {
			let mut tx1 = tx0.deref().clone();
			tx1.nonce = U256::from(124);
			tx1.sign(&keypair.secret())
		};
		let tx2 = {
			let mut tx2 = tx1.deref().clone();
			tx2.gas_price = U256::from(200);
			tx2.sign(&keypair.secret())
		};

		// when
		txq.add(tx1, &default_nonce).unwrap();
		txq.add(tx2, &default_nonce).unwrap();
		assert_eq!(txq.status().future, 1);
		txq.add(tx0, &default_nonce).unwrap();

		// then
		let stats = txq.status();
		assert_eq!(stats.future, 0);
		assert_eq!(stats.pending, 2);
		assert_eq!(txq.top_transactions()[1].gas_price, U256::from(200));
	}

	#[test]
	fn should_recalculate_height_when_removing_from_future() {
		// given
		let previous_nonce = |a: &Address| AccountDetails{ nonce: default_nonce(a).nonce - U256::one(), balance:
			!U256::zero() };
		let next_nonce = |a: &Address| AccountDetails{ nonce: default_nonce(a).nonce + U256::one(), balance:
			!U256::zero() };
		let mut txq = TransactionQueue::new();
		let (tx1, tx2) = new_txs(U256::one());
		txq.add(tx1.clone(), &previous_nonce).unwrap();
		txq.add(tx2, &previous_nonce).unwrap();
		assert_eq!(txq.status().future, 2);

		// when
		txq.remove_invalid(&tx1.hash(), &next_nonce);

		// then
		let stats = txq.status();
		assert_eq!(stats.future, 0);
		assert_eq!(stats.pending, 1);
	}

	#[test]
	fn should_return_none_when_transaction_from_given_address_does_not_exist() {
		// given
		let txq = TransactionQueue::new();

		// then
		assert_eq!(txq.last_nonce(&Address::default()), None);
	}

	#[test]
	fn should_return_correct_nonce_when_transactions_from_given_address_exist() {
		// given
		let mut txq = TransactionQueue::new();
		let tx = new_tx();
		let from = tx.sender().unwrap();
		let nonce = tx.nonce;
		let details = |_a: &Address| AccountDetails { nonce: nonce, balance: !U256::zero() };

		// when
		txq.add(tx, &details).unwrap();

		// then
		assert_eq!(txq.last_nonce(&from), Some(nonce));
	}

	#[test]
	fn should_remove_old_transaction_even_if_newer_transaction_was_not_known() {
		// given
		let mut txq = TransactionQueue::new();
		let (tx1, tx2) = new_txs(U256::one());
		let (nonce1, nonce2) = (tx1.nonce, tx2.nonce);
		let details1 = |_a: &Address| AccountDetails { nonce: nonce1, balance: !U256::zero() };

		// Insert first transaction
		txq.add(tx1, &details1).unwrap();

		// when
		txq.remove_all(tx2.sender().unwrap(), nonce2 + U256::one());

		// then
		assert!(txq.top_transactions().is_empty());
	}
}
