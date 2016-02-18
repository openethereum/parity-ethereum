

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

//! A set of utilities to abstract tasks execution.
//!
//! Tasks can be executed in the same thread, multiple threads (pool) or you can
//! control when and in which thread the tasks are consumed.

extern crate eventual;
use self::eventual::Complete;
pub use self::eventual::{Future, Async};

use std::boxed::FnBox;
use std::cmp::{Ord, Ordering};
use std::collections::binary_heap::BinaryHeap;
use std::marker::PhantomData;
use std::sync::{atomic, Arc, Condvar, Mutex};
use std::thread::{JoinHandle, spawn};

#[macro_export]
macro_rules! trivial_ordering {
	($t:ident by $cmp:expr) => {
		impl Eq for $t {}
		impl PartialEq for $t {
			fn eq(&self, other: &$t) -> bool {
				self.cmp(other) == Ordering::Equal
			}
		}
		impl Ord for $t {
			fn cmp(&self, other: &$t) -> Ordering {
				$cmp(self, other)
			}
		}
		impl PartialOrd for $t {
			fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
				Some(self.cmp(other))
			}
		}
	}
}

/// Some work to be done in a thread pool
pub trait Task: Send + 'static {
	/// Task result
	type Result: Send + 'static;
	/// Task error
	type Error: Send + 'static;

	/// Actual computation to be done
	fn call(self) -> Result<Self::Result, Self::Error>;
}

/// Abstraction over task execution
pub trait Executor<T : Task> {
	/// Returns number of elements waiting in queue
	fn queued(&self) -> usize;
	/// Add new task to be executed. Lower value in `priority` means faster execution. Returns a `Future` result.
	fn execute_with_priority(&self, task: T, priority: usize) -> Future<T::Result, T::Error>;
	/// Add new task to be executed with default priority `std::usize::MAX / 2`
	fn execute(&self, task: T) -> Future<T::Result, T::Error> {
		self.execute_with_priority(task, (!0 as usize) / 2)
	}

	/// Clear any pending tasks
	fn clear(&self);
}

/// PriorityQueue of tasks waiting for execution
pub trait TaskQueue<T: Task>: Send + 'static {
	/// Add new element to this queue
	fn push(&mut self, elem: TaskQueueElement<T>);
	/// Try to get `Task` that should be executed next.
	/// Returns `None` if no more work is waiting in queue.
	fn try_next(&mut self) -> Option<TaskQueueElement<T>>;
	/// Returns the size of the queue
	fn len(&self) -> usize;
	/// Removes all elements from the queue
	fn clear(&mut self);
	/// Returns true if there are no items in queue
	fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

/// Single element inside a `TaskQueue`. Represents
pub struct TaskQueueElement<T: Task> {
	/// Task
	task: T,
	/// Priority
	priority: usize,
	/// Task fulfillment or rejection handler
	complete: Complete<T::Result, T::Error>
}

/// Type for tasks that are just closures
pub struct ClosureTask<R, E>
	where R: Send + 'static,
		  E: Send + 'static {
	closure: Box<FnBox() -> Result<R, E> + Send>,
}

// Implementations
/// Utility structure to create possible executors.
pub struct Executors;
impl Executors {

	/// Creates new `Task` from a closure.
	///
	/// Essential to have type correctness without implementing `Task` trait on ones own.
	pub fn task<R, E, F>(closure: F) -> ClosureTask<R, E>
		where F: FnBox() -> Result<R, E> + Send + 'static,
			  R: Send + 'static,
			  E: Send + 'static {
		ClosureTask {
			closure: Box::new(closure)
		}
	}

	/// Create executor that runs tasks synchronously.
	pub fn same_thread() -> SameThreadExecutor {
		SameThreadExecutor
	}

	/// Create executor that uses thread pool and priority queue to run tasks
	pub fn thread_pool<T>(threads: usize) -> ThreadPoolExecutor<T, BinaryHeap<TaskQueueElement<T>>>
		where T: Task {
			let queue = BinaryHeap::new();
			ThreadPoolExecutor::new(threads, queue)
		}

	/// Create executor with manual control over when the tasks are executed
	pub fn manual<T>() -> ManualExecutor<T, BinaryHeap<TaskQueueElement<T>>>
		where T: Task {
			let queue = BinaryHeap::new();
			ManualExecutor::new(queue)
		}

}

/// Executor with thread pool
pub struct ThreadPoolExecutor<T, Queue>
	where T: Task, Queue: TaskQueue<T> {

	wait: Arc<Condvar>,
	finished: Arc<atomic::AtomicBool>,
	queue: Arc<Mutex<Queue>>,
	threads: Vec<JoinHandle<()>>,
	_task: PhantomData<T>
}

impl<T, Queue> ThreadPoolExecutor<T, Queue>
	where T: Task, Queue: TaskQueue<T> {

	/// Creates new `ThreadPoolExecutor` with specified number of threads in pool and custom `Queue` implementation.
	fn new(threads_num: usize, queue: Queue) -> ThreadPoolExecutor<T, Queue> {
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

	fn worker(wait: Arc<Condvar>, finished: Arc<atomic::AtomicBool>, queue: Arc<Mutex<Queue>>) {
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

impl<T, Queue> Drop for ThreadPoolExecutor<T, Queue>
	where T: Task, Queue: TaskQueue<T> {
	fn drop(&mut self) {
		self.finished.store(true, atomic::Ordering::Relaxed);
		self.wait.notify_all();

		for thread in self.threads.drain(..) {
			thread.join().unwrap();
		}
	}
}

impl<T, Queue> Executor<T> for ThreadPoolExecutor<T, Queue>
	where T: Task, Queue: TaskQueue<T> {

	fn queued(&self) -> usize {
		let q = self.queue.lock().unwrap();
		q.len()
	}

	fn clear(&self) {
		let mut q = self.queue.lock().unwrap();
		q.clear();
	}

	fn execute_with_priority(&self, task: T, priority: usize) -> Future<T::Result, T::Error> {
		let (complete, future) = Future::pair();
		let mut q = self.queue.lock().unwrap();
		q.push(TaskQueueElement {
			task: task,
			priority: priority,
			complete: complete,
		});
		self.wait.notify_one();
		future
	}
}

/// Synchronous executor implementation
pub struct SameThreadExecutor;
impl<T: Task> Executor<T> for SameThreadExecutor {

	fn queued(&self) -> usize {
		// Everything is run synchronously
		0
	}

	fn clear(&self) {
		// There is no queue
	}

	fn execute_with_priority(&self, task: T, _priority: usize) -> Future<T::Result, T::Error> {
		task.call()
			.map(Future::of)
			.unwrap_or_else(Future::error)
	}
}

/// Manual executor. You can add tasks to it but you need to manually specify when the tasks should be invoked.
pub struct ManualExecutor<T, Queue>
	where T: Task, Queue: TaskQueue<T> {
	queue: Mutex<Queue>,
	_task: PhantomData<T>
}

impl<T, Queue> ManualExecutor<T, Queue>
	where T: Task, Queue: TaskQueue<T> {

	/// Returns new `ManualExecutor` with custom `Queue` implementation
	fn new(queue: Queue) -> ManualExecutor<T, Queue> {
		ManualExecutor {
			queue: Mutex::new(queue),
			_task: PhantomData
		}
	}

	/// Execute specified number of tasks from queue.
	///
	/// Panics when requested to consume more than there is in queue.
	pub fn consume(&self, amount: usize) {
		let mut queue = self.queue.lock().unwrap();
		for _i in 0..amount {
			let work = queue.try_next().expect("Not enough items to consume.");
			let result = work.consume();
			match result {
				(complete, Ok(res)) => complete.complete(res),
				(complete, Err(err)) => complete.fail(err)
			}
		}
	}
}
impl<T, Queue> Executor<T> for ManualExecutor<T, Queue>
	where T: Task, Queue: TaskQueue<T> {

	fn queued(&self) -> usize {
		let q = self.queue.lock().unwrap();
		q.len()
	}

	fn clear(&self) {
		let mut q = self.queue.lock().unwrap();
		q.clear();
	}

	fn execute_with_priority(&self, task: T, priority: usize) -> Future<T::Result, T::Error> {
		let (complete, future) = Future::pair();
		let mut q = self.queue.lock().unwrap();
		q.push(TaskQueueElement {
			task: task,
			priority: priority,
			complete: complete,
		});
		future
	}
}

// ClosureTask
impl<R, E> Task for ClosureTask<R, E>
	where R: Send + 'static,
		  E: Send + 'static {
	type Result = R;
	type Error = E;

	fn call(self) -> Result<Self::Result, Self::Error> {
		(self.closure)()
	}
}

// TaskQueueElement ordering
// TODO [todr] This implementation is only valid for structures that allows duplicated elements.
// T1=T2 if T1.priority = T2.priority
impl<T: Task> TaskQueueElement<T> {
	fn consume(self) -> (Complete<T::Result, T::Error>, Result<T::Result, T::Error>) {
		(self.complete, self.task.call())
	}
}
impl<T: Task> Ord for TaskQueueElement<T> {
	fn cmp(&self, other: &TaskQueueElement<T>) -> Ordering {
		self.priority.cmp(&other.priority)
	}
}
impl<T: Task> PartialOrd for TaskQueueElement<T> {
	fn partial_cmp(&self, other: &TaskQueueElement<T>) -> Option<Ordering> {
		self.priority.partial_cmp(&other.priority)
	}
}
impl<T: Task> Eq for TaskQueueElement<T> {}
impl<T: Task> PartialEq for TaskQueueElement<T> {
	fn eq(&self, other: &TaskQueueElement<T>) -> bool {
		self.priority.eq(&other.priority)
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

	fn clear(&mut self) {
		BinaryHeap::clear(self)
	}

	fn len(&self) -> usize {
		BinaryHeap::len(self)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn should_execute_task() {
		// given
		let e = Executors::same_thread();

		// when
		let future: Future<usize, ()> = e.execute(Executors::task(|| Ok(4)));

		// then
		assert_eq!(4, future.await().unwrap());
	}

	#[test]
	fn should_execute_task_in_thread_pool() {
		// given
		let e = Executors::thread_pool(1);

		// when
		let future: Future<usize, ()> = e.execute(Executors::task(|| Ok(4)));

		// then
		assert_eq!(4, future.await().unwrap());
	}

	#[test]
	fn should_execute_task_in_test_executor() {
		// given
		let e = Executors::test_executor();

		// when
		let future = e.execute(Executors::task(|| Ok(4)));
		let future2 = e.execute(Executors::task(|| Ok(5)));
		let future3 = e.execute(Executors::task(|| Err(6)));
		assert_eq!(3, e.queued());
		e.consume(3);

		// then
		assert_eq!(4, future.await().unwrap());
		assert_eq!(5, future2.await().unwrap());
		assert_eq!(6, future3.await().unwrap_err().unwrap());
	}
}
