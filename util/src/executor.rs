

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

extern crate eventual;
pub use self::eventual::*;

use std::cmp::{Ord, Ordering};
use std::collections::binary_heap::BinaryHeap;
use std::marker::PhantomData;
use std::sync::{atomic, Arc, Condvar, Mutex};
use std::thread::{JoinHandle, spawn};

#[macro_export]
macro_rules! ordering_for {
	($type_: ident using $field_: ident) => {
		impl Ord for $type_ {
			fn cmp(&self, other: &$type_) -> Ordering {
				self.$field_.cmp(&other.$field_)
			}
		}
		impl PartialOrd for $type_ {
			fn partial_cmp(&self, other: &$type_) -> Option<Ordering> {
				self.$field_.partial_cmp(&other.$field_)
			}
		}
		impl Eq for $type_ {}
		impl PartialEq for $type_ {
			fn eq(&self, other: &$type_) -> bool {
				self.$field_.eq(&other.$field_)
			}
		}
	}
}

/// Some work to be done in a thread pool
pub trait Task: Ord + Send + 'static {
	/// Task result
	type TResult: Send + 'static;
	/// Task error
	type TError: Send + 'static;

	/// Actual computation to be done
	fn call(&mut self) -> Result<Self::TResult, Self::TError>;
}

/// Abstraction over task execution
pub trait Executor<T : Task> {
	/// Add new task to be executed. Returns a `Future` result.
	fn execute(&mut self, task: T) -> Future<T::TResult, T::TError>;
}

/// PriorityQueue of tasks waiting for execution
pub trait TaskQueue<T: Task>: Send + 'static {
	/// Add new element to this queue
	fn push(&mut self, elem: TaskQueueElement<T>);
	/// Try to get `Task` that should be executed next.
	/// Returns `None` if no more work is waiting in queue.
	fn try_next(&mut self) -> Option<TaskQueueElement<T>>;
}

/// Single element inside a `TaskQueue`. Represents
pub struct TaskQueueElement<T: Task> {
	/// Task
	task: T,
	/// Task fulfillment or rejection handler
	complete: Complete<T::TResult, T::TError>
}

// Implementations
pub struct Executors;
impl Executors {
	pub fn same_thread() -> SameThreadExecutor {
		SameThreadExecutor
	}

	pub fn thread_pool<TTask, TQueue>(threads: usize, queue: TQueue) -> ThreadPoolExecutor<TTask, TQueue>
		where TTask: Task, TQueue : TaskQueue<TTask> {
			ThreadPoolExecutor::new(threads, queue)
		}
}

pub struct ThreadPoolExecutor<TTask, TQueue>
	where TTask: Task, TQueue: TaskQueue<TTask> {

	wait: Arc<Condvar>,
	finished: Arc<atomic::AtomicBool>,
	queue: Arc<Mutex<TQueue>>,
	threads: Vec<JoinHandle<()>>,
	_task: PhantomData<TTask>
}

impl<TTask, TQueue> ThreadPoolExecutor<TTask, TQueue>
	where TTask: Task, TQueue: TaskQueue<TTask> {

	pub fn new(threads_num: usize, queue: TQueue) -> ThreadPoolExecutor<TTask, TQueue> {
		assert!(threads_num > 0);

		let wait = Arc::new(Condvar::new());
		let finished = Arc::new(atomic::AtomicBool::new(false));
		let queue = Arc::new(Mutex::new(queue));
		let mut threads = vec![];

		for _i in 0..threads_num {
			let wait = wait.clone();
			let finished = finished.clone();
			let queue = queue.clone();
			threads.push(spawn(move || ThreadPoolExecutor::worker(wait, finished, queue)));
		}

		ThreadPoolExecutor {
			wait: wait,
			finished: finished,
			queue: queue,
			threads: threads,
			_task: PhantomData
		}
	}

	fn worker(wait: Arc<Condvar>, finished: Arc<atomic::AtomicBool>, queue: Arc<Mutex<TQueue>>) {
		loop {
			// Initial check if not finished
			if finished.load(atomic::Ordering::Relaxed) {
				return;
			}

			let mut queue = queue.lock().unwrap();
			// Do work if any
			if let Some(work) = queue.try_next() {
				let result = work.consume();
				match result {
					(complete, Ok(res)) => complete.complete(res),
					(complete, Err(err)) => complete.fail(err)
				}
			}

			// Thread might be finished by the time we get here
			if finished.load(atomic::Ordering::Relaxed) {
				return;
			}

			// Wait for some more work
			let _ = wait.wait(queue).unwrap();
		}
	}
}

impl<TTask, TQueue> Drop for ThreadPoolExecutor<TTask, TQueue>
	where TTask: Task, TQueue: TaskQueue<TTask> {
	fn drop(&mut self) {
		self.finished.store(true, atomic::Ordering::Relaxed);
		self.wait.notify_all();

		for thread in self.threads.drain(..) {
			thread.join().unwrap();
		}
	}
}

impl<TTask, TQueue> Executor<TTask> for ThreadPoolExecutor<TTask, TQueue>
	where TTask: Task, TQueue: TaskQueue<TTask> {

	fn execute(&mut self, task: TTask) -> Future<TTask::TResult, TTask::TError> {
		let (complete, future) = Future::pair();
		let mut q = self.queue.lock().unwrap();
		q.push(TaskQueueElement {
			task: task,
			complete: complete,
		});
		self.wait.notify_one();
		future
	}
}

pub struct SameThreadExecutor;
impl<TTask: Task> Executor<TTask> for SameThreadExecutor {

	fn execute(&mut self, mut task: TTask) -> Future<TTask::TResult, TTask::TError> {
		task.call()
			.map(Future::of)
			.unwrap_or_else(Future::error)
	}
}

// TaskQueueElement ordering
impl<T: Task> TaskQueueElement<T> {
	fn consume(self) -> (Complete<T::TResult, T::TError>, Result<T::TResult, T::TError>) {
		let mut task = self.task;
		(self.complete, task.call())
	}
}
impl<T: Task> Ord for TaskQueueElement<T> {
	fn cmp(&self, other: &TaskQueueElement<T>) -> Ordering {
		self.task.cmp(&other.task)
	}
}
impl<T: Task> PartialOrd for TaskQueueElement<T> {
	fn partial_cmp(&self, other: &TaskQueueElement<T>) -> Option<Ordering> {
		self.task.partial_cmp(&other.task)
	}
}
impl<T: Task> Eq for TaskQueueElement<T> {}
impl<T: Task> PartialEq for TaskQueueElement<T> {
	fn eq(&self, other: &TaskQueueElement<T>) -> bool {
		self.task.eq(&other.task)
	}
}

// TaskQueue implementation for BinaryHeap
impl<T: Task> TaskQueue<T> for BinaryHeap<TaskQueueElement<T>> {
	fn push(&mut self, elem: TaskQueueElement<T>) {
		BinaryHeap::push(self, elem);
	}

	fn try_next(&mut self) -> Option<TaskQueueElement<T>> {
		self.pop()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::cmp::Ordering;
	use std::collections::binary_heap::BinaryHeap;

	pub struct TestTask {
		priority: usize,
		closure: Box<FnMut() -> Result<usize, ()> + Send + Sync>,
	}

	ordering_for!(TestTask using priority);

	impl TestTask {
		fn new<T>(closure: T) -> TestTask where T: FnMut() -> Result<usize, ()> + Send + Sync + 'static {
			TestTask {
				priority: 1,
				closure: Box::new(closure)
			}
		}
	}

	impl Task for TestTask {
		type TResult = usize;
		type TError = ();

		fn call(&mut self) -> Result<Self::TResult, Self::TError>{
			(self.closure)()
		}
	}

	#[test]
	fn should_execute_task() {
		// given
		let mut e = Executors::same_thread();

		// when
		let future = e.execute(TestTask::new(|| Ok(4)));

		// then
		assert_eq!(4, future.await().unwrap());
	}

	#[test]
	fn should_execute_task_in_thread_pool() {
		// given
		let queue = BinaryHeap::new();
		let mut e = Executors::thread_pool(1, queue);

		// when
		let future = e.execute(TestTask::new(|| Ok(4)));

		// then
		assert_eq!(4, future.await().unwrap());
	}
}
