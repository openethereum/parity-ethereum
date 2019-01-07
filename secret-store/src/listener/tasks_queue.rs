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

use std::collections::VecDeque;
use parking_lot::{Mutex, Condvar};

#[derive(Default)]
/// General deque-based tasks queue.
pub struct TasksQueue<Task: Clone> {
	/// Service event.
	service_event: Condvar,
	/// Service tasks queue.
	service_tasks: Mutex<VecDeque<Task>>,
}

impl<Task> TasksQueue<Task> where Task: Clone {
	/// Create new tasks queue.
	pub fn new() -> Self {
		TasksQueue {
			service_event: Condvar::new(),
			service_tasks: Mutex::new(VecDeque::new()),
		}
	}

	#[cfg(test)]
	/// Get current tasks snapshot.
	pub fn snapshot(&self) -> VecDeque<Task> {
		self.service_tasks.lock().clone()
	}

	/// Push task to the front of queue.
	pub fn push_front(&self, task: Task) {
		let mut service_tasks = self.service_tasks.lock();
		service_tasks.push_front(task);
		self.service_event.notify_all();
	}

	/// Push task to the back of queue.
	pub fn push(&self, task: Task) {
		let mut service_tasks = self.service_tasks.lock();
		service_tasks.push_back(task);
		self.service_event.notify_all();
	}

	/// Push task to the back of queue.
	pub fn push_many<I: Iterator<Item=Task>>(&self, tasks: I) {
		let mut service_tasks = self.service_tasks.lock();
		let previous_len = service_tasks.len();
		service_tasks.extend(tasks);
		if service_tasks.len() != previous_len {
			self.service_event.notify_all();
		}
	}

	/// Wait for new task (task is removed from the front of queue).
	pub fn wait(&self) -> Task {
		let mut service_tasks = self.service_tasks.lock();
		if service_tasks.is_empty() {
			self.service_event.wait(&mut service_tasks);
		}

		service_tasks.pop_front()
			.expect("service_event is only fired when there are new tasks; qed")
	}
}
