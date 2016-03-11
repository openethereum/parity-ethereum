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

//! Indexes all rpc poll requests.

use util::hash::H256;
use std::collections::HashMap;
use transient_hashmap::{TransientHashMap, Timer, StandardTimer};

/// Lifetime of poll (in seconds).
const POLL_LIFETIME: u64 = 60;

pub type PollId = usize;
pub type BlockNumber = u64;

pub struct PollInfo<F> {
	pub filter: F,
	pub block_number: BlockNumber
}

impl<F> Clone for PollInfo<F> where F: Clone {
	fn clone(&self) -> Self {
		PollInfo {
			filter: self.filter.clone(),
			block_number: self.block_number.clone()
		}
	}
}

/// Indexes all poll requests.
///
/// Lazily garbage collects unused polls info.
pub struct PollManager<F, T = StandardTimer> where T: Timer {
	polls: TransientHashMap<PollId, PollInfo<F>, T>,
	transactions_data: HashMap<PollId, Vec<H256>>,
	next_available_id: PollId,
}

impl<F> PollManager<F, StandardTimer> {
	/// Creates new instance of indexer.
	pub fn new() -> Self {
		PollManager::new_with_timer(Default::default())
	}
}

impl<F, T> PollManager<F, T> where T: Timer {
	pub fn new_with_timer(timer: T) -> Self {
		PollManager {
			polls: TransientHashMap::new_with_timer(POLL_LIFETIME, timer),
			transactions_data: HashMap::new(),
			next_available_id: 0,
		}
	}

	fn prune(&mut self) {
		self.polls.prune();
		// self.polls.prune()
		// 	.into_iter()
		// 	.map(|key| {
		// 		self.transactions_data.remove(key);
		// 	});
	}

	/// Returns id which can be used for new poll.
	///
	/// Stores information when last poll happend.
	pub fn create_poll(&mut self, filter: F, block: BlockNumber) -> PollId {
		self.prune();
		let id = self.next_available_id;
		self.next_available_id += 1;
		self.polls.insert(id, PollInfo {
			filter: filter,
			block_number: block,
		});
		id
	}

	/// Updates information when last poll happend.
	pub fn update_poll(&mut self, id: &PollId, block: BlockNumber) {
		self.prune();
		if let Some(info) = self.polls.get_mut(id) {
			info.block_number = block;
		}
	}

	/// Returns number of block when last poll happend.
	pub fn poll_info(&mut self, id: &PollId) -> Option<&PollInfo<F>> {
		self.prune();
		self.polls.get(id)
	}

	pub fn update_transactions(&mut self, id: &PollId, transactions: Vec<H256>) -> Option<Vec<H256>> {
		self.prune();
		if self.polls.get(id).is_some() {
			self.transactions_data.insert(*id, transactions)
		} else {
			None
		}
	}

	// Normal code always replaces transactions
	#[cfg(test)]
	/// Returns last transactions hashes for given poll.
	pub fn transactions(&mut self, id: &PollId) -> Option<&Vec<H256>> {
		self.prune();
		self.transactions_data.get(id)
	}

	/// Removes poll info.
	pub fn remove_poll(&mut self, id: &PollId) {
		self.polls.remove(id);
		self.transactions_data.remove(id);
	}
}

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use transient_hashmap::Timer;
	use v1::helpers::PollManager;
	use util::hash::H256;

	struct TestTimer<'a> {
		time: &'a RefCell<i64>,
	}

	impl<'a> Timer for TestTimer<'a> {
		fn get_time(&self) -> i64 {
			*self.time.borrow()
		}
	}

	#[test]
	fn test_poll_indexer() {
		let time = RefCell::new(0);
		let timer = TestTimer {
			time: &time,
		};

		let mut indexer = PollManager::new_with_timer(timer);
		assert_eq!(indexer.create_poll(false, 20), 0);
		assert_eq!(indexer.create_poll(true, 20), 1);

		*time.borrow_mut() = 10;
		indexer.update_poll(&0, 21);
		assert_eq!(indexer.poll_info(&0).unwrap().filter, false);
		assert_eq!(indexer.poll_info(&0).unwrap().block_number, 21);

		*time.borrow_mut() = 30;
		indexer.update_poll(&1, 23);
		assert_eq!(indexer.poll_info(&1).unwrap().filter, true);
		assert_eq!(indexer.poll_info(&1).unwrap().block_number, 23);

		*time.borrow_mut() = 75;
		indexer.update_poll(&0, 30);
		assert!(indexer.poll_info(&0).is_none());
		assert_eq!(indexer.poll_info(&1).unwrap().filter, true);
		assert_eq!(indexer.poll_info(&1).unwrap().block_number, 23);

		indexer.remove_poll(&1);
		assert!(indexer.poll_info(&1).is_none());
	}

	#[test]
	fn should_return_poll_transactions_hashes() {
		// given
		let mut indexer = PollManager::new();
		let poll_id = indexer.create_poll(false, 20);
		assert!(indexer.transactions(&poll_id).is_none());
		let transactions = vec![H256::from(1), H256::from(2)];

		// when
		indexer.update_transactions(&poll_id, transactions.clone());

		// then
		let txs = indexer.transactions(&poll_id);
		assert_eq!(txs.unwrap(), &transactions);
	}


	#[test]
	fn should_remove_transaction_data_when_poll_timed_out() {
		// given
		let time = RefCell::new(0);
		let timer = TestTimer {
			time: &time,
		};
		let mut indexer = PollManager::new_with_timer(timer);
		let poll_id = indexer.create_poll(false, 20);
		let transactions = vec![H256::from(1), H256::from(2)];
		indexer.update_transactions(&poll_id, transactions.clone());
		assert!(indexer.transactions(&poll_id).is_some());

		// when
		*time.borrow_mut() = 75;
		indexer.prune();

		// then
		assert!(indexer.transactions(&poll_id).is_none());

	}

	#[test]
	fn should_remove_transaction_data_when_poll_is_removed() {
		// given
		let mut indexer = PollManager::new();
		let poll_id = indexer.create_poll(false, 20);
		let transactions = vec![H256::from(1), H256::from(2)];

		// when
		indexer.update_transactions(&poll_id, transactions.clone());
		assert!(indexer.transactions(&poll_id).is_some());
		indexer.remove_poll(&poll_id);

		// then
		assert!(indexer.transactions(&poll_id).is_none());
	}

	#[test]
	fn should_ignore_transactions_for_invalid_poll_id() {
		// given
		let mut indexer = PollManager::<()>::new();
		let transactions = vec![H256::from(1), H256::from(2)];

		// when
		indexer.update_transactions(&5, transactions.clone());

		// then
		assert!(indexer.transactions(&5).is_none());
	}
}
