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
	next_available_id: PollId
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
			next_available_id: 0
		}
	}

	/// Returns id which can be used for new poll.
	///
	/// Stores information when last poll happend.
	pub fn create_poll(&mut self, filter: F, block: BlockNumber) -> PollId {
		self.polls.prune();
		let id = self.next_available_id;
		self.next_available_id += 1;
		self.polls.insert(id, PollInfo {
			filter: filter,
			block_number: block
		});
		id
	}

	/// Updates information when last poll happend.
	pub fn update_poll(&mut self, id: &PollId, block: BlockNumber) {
		self.polls.prune();
		if let Some(info) = self.polls.get_mut(id) {
			info.block_number = block;
		}
	}

	/// Returns number of block when last poll happend.
	pub fn get_poll_info(&mut self, id: &PollId) -> Option<&PollInfo<F>> {
		self.polls.prune();
		self.polls.get(id)
	}

	/// Removes poll info.
	pub fn remove_poll(&mut self, id: &PollId) {
		self.polls.remove(id);
	}
}

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use transient_hashmap::Timer;
	use v1::helpers::PollManager;

	struct TestTimer<'a> {
		time: &'a RefCell<i64>
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
			time: &time
		};

		let mut indexer = PollManager::new_with_timer(timer);
		assert_eq!(indexer.create_poll(false, 20), 0);
		assert_eq!(indexer.create_poll(true, 20), 1);

		*time.borrow_mut() = 10;
		indexer.update_poll(&0, 21);
		assert_eq!(indexer.get_poll(&0).unwrap().filter, false);
		assert_eq!(indexer.get_poll(&0).unwrap().block_number, 21);

		*time.borrow_mut() = 30;
		indexer.update_poll(&1, 23);
		assert_eq!(indexer.get_poll(&1).unwrap().filter, true);
		assert_eq!(indexer.get_poll(&1).unwrap().block_number, 23);

		*time.borrow_mut() = 75;
		indexer.update_poll(&0, 30);
		assert!(indexer.get_poll(&0).is_none());
		assert_eq!(indexer.get_poll(&1).unwrap().filter, true);
		assert_eq!(indexer.get_poll(&1).unwrap().block_number, 23);

		indexer.remove_poll(&1);
		assert!(indexer.get_poll(&1).is_none());
	}
}
