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

/// Indexes all poll requests.
///
/// Lazily garbage collects unused polls info.
pub struct PollManager<F, T = StandardTimer> where T: Timer {
	polls: TransientHashMap<PollId, F, T>,
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
			next_available_id: 0,
		}
	}

	/// Returns id which can be used for new poll.
	///
	/// Stores information when last poll happend.
	pub fn create_poll(&mut self, filter: F) -> PollId {
		self.polls.prune();

		let id = self.next_available_id;
		self.polls.insert(id, filter);

		self.next_available_id += 1;
		id
	}

	// Implementation is always using `poll_mut`
	/// Get a reference to stored poll filter
	pub fn poll(&mut self, id: &PollId) -> Option<&F> {
		self.polls.prune();
		self.polls.get(id)
	}

	/// Get a mutable reference to stored poll filter
	pub fn poll_mut(&mut self, id: &PollId) -> Option<&mut F> {
		self.polls.prune();
		self.polls.get_mut(id)
	}

	/// Removes poll info.
	pub fn remove_poll(&mut self, id: &PollId) {
		self.polls.remove(id);
	}
}

#[cfg(test)]
mod tests {
	use std::cell::Cell;
	use transient_hashmap::Timer;
	use v1::helpers::PollManager;

	struct TestTimer<'a> {
		time: &'a Cell<i64>,
	}

	impl<'a> Timer for TestTimer<'a> {
		fn get_time(&self) -> i64 {
			self.time.get()
		}
	}

	#[test]
	fn test_poll_indexer() {
		let time = Cell::new(0);
		let timer = TestTimer {
			time: &time,
		};

		let mut indexer = PollManager::new_with_timer(timer);
		assert_eq!(indexer.create_poll(20), 0);
		assert_eq!(indexer.create_poll(20), 1);

		time.set(10);
		*indexer.poll_mut(&0).unwrap() = 21;
		assert_eq!(*indexer.poll(&0).unwrap(), 21);
		assert_eq!(*indexer.poll(&1).unwrap(), 20);

		time.set(30);
		*indexer.poll_mut(&1).unwrap() = 23;
		assert_eq!(*indexer.poll(&1).unwrap(), 23);

		time.set(75);
		assert!(indexer.poll(&0).is_none());
		assert_eq!(*indexer.poll(&1).unwrap(), 23);

		indexer.remove_poll(&1);
		assert!(indexer.poll(&1).is_none());
	}

}
