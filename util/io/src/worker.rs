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

use std::{
	sync::{Arc, atomic::{AtomicBool, Ordering as AtomicOrdering}},
	thread::{self, JoinHandle},
};

use crossbeam_deque as deque;
use futures::future::{self, Loop};
use log::{trace, error};
use parking_lot::{Condvar, Mutex};
use tokio;

use crate::{
	IoHandler,
	LOCAL_STACK_SIZE,
	service_mio::{HandlerId, IoChannel, IoContext},
};

const STACK_SIZE: usize = 16*1024*1024;

pub enum WorkType<Message> {
	Readable,
	Writable,
	Hup,
	Timeout,
	Message(Arc<Message>)
}

pub struct Work<Message> {
	pub work_type: WorkType<Message>,
	pub token: usize,
	pub handler_id: HandlerId,
	pub handler: Arc<dyn IoHandler<Message>>,
}

/// An IO worker thread
/// Sorts them ready for blockchain insertion.
pub struct Worker {
	thread: Option<JoinHandle<()>>,
	wait: Arc<Condvar>,
	deleting: Arc<AtomicBool>,
	wait_mutex: Arc<Mutex<()>>,
}

impl Worker {
	/// Creates a new worker instance.
	pub fn new<Message>(
		index: usize,
		stealer: deque::Stealer<Work<Message>>,
		channel: IoChannel<Message>,
		wait: Arc<Condvar>,
		wait_mutex: Arc<Mutex<()>>,
	) -> Worker
	where Message: Send + Sync + 'static
	{
		let deleting = Arc::new(AtomicBool::new(false));
		let mut worker = Worker {
			thread: None,
			wait: wait.clone(),
			deleting: deleting.clone(),
			wait_mutex: wait_mutex.clone(),
		};
		worker.thread = Some(thread::Builder::new().stack_size(STACK_SIZE).name(format!("IO Worker #{}", index)).spawn(
			move || {
				LOCAL_STACK_SIZE.with(|val| val.set(STACK_SIZE));
				let ini = (stealer, channel.clone(), wait, wait_mutex.clone(), deleting);
				let future = future::loop_fn(ini, |(stealer, channel, wait, wait_mutex, deleting)| {
					{
						let mut lock = wait_mutex.lock();
						if deleting.load(AtomicOrdering::Acquire) {
							return Ok(Loop::Break(()));
						}
						wait.wait(&mut lock);
					}

					while !deleting.load(AtomicOrdering::Acquire) {
						match stealer.steal() {
							deque::Steal::Data(work) => Worker::do_work(work, channel.clone()),
							deque::Steal::Retry => {},
							deque::Steal::Empty => break,
						}
					}
					Ok(Loop::Continue((stealer, channel, wait, wait_mutex, deleting)))
				});
				if let Err(()) = tokio::runtime::current_thread::block_on_all(future) {
					error!(target: "ioworker", "error while executing future")
				}
			})
			.expect("Error creating worker thread"));
		worker
	}

	fn do_work<Message>(work: Work<Message>, channel: IoChannel<Message>) where Message: Send + Sync + 'static {
		match work.work_type {
			WorkType::Readable => {
				work.handler.stream_readable(&IoContext::new(channel, work.handler_id), work.token);
			},
			WorkType::Writable => {
				work.handler.stream_writable(&IoContext::new(channel, work.handler_id), work.token);
			}
			WorkType::Hup => {
				work.handler.stream_hup(&IoContext::new(channel, work.handler_id), work.token);
			}
			WorkType::Timeout => {
				work.handler.timeout(&IoContext::new(channel, work.handler_id), work.token);
			}
			WorkType::Message(message) => {
				work.handler.message(&IoContext::new(channel, work.handler_id), &*message);
			}
		}
	}
}

impl Drop for Worker {
	fn drop(&mut self) {
		trace!(target: "shutdown", "[IoWorker] Closing...");
		let _ = self.wait_mutex.lock();
		self.deleting.store(true, AtomicOrdering::Release);
		self.wait.notify_all();
		if let Some(thread) = self.thread.take() {
			thread.join().ok();
		}
		trace!(target: "shutdown", "[IoWorker] Closed");
	}
}
