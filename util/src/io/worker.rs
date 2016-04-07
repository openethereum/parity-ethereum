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

use std::sync::*;
use std::mem;
use std::thread::{JoinHandle, self};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use crossbeam::sync::chase_lev;
use io::service::{HandlerId, IoChannel, IoContext};
use io::{IoHandler};
use panics::*;

pub enum WorkType<Message> {
	Readable,
	Writable,
	Hup,
	Timeout,
	Message(Message)
}

pub struct Work<Message> {
	pub work_type: WorkType<Message>,
	pub token: usize,
	pub handler_id: HandlerId,
	pub handler: Arc<IoHandler<Message>>,
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
	pub fn new<Message>(index: usize,
						stealer: chase_lev::Stealer<Work<Message>>,
						channel: IoChannel<Message>,
						wait: Arc<Condvar>,
						wait_mutex: Arc<Mutex<()>>,
						panic_handler: Arc<PanicHandler>
					   ) -> Worker
					where Message: Send + Sync + Clone + 'static {
		let deleting = Arc::new(AtomicBool::new(false));
		let mut worker = Worker {
			thread: None,
			wait: wait.clone(),
			deleting: deleting.clone(),
			wait_mutex: wait_mutex.clone(),
		};
		worker.thread = Some(thread::Builder::new().name(format!("IO Worker #{}", index)).spawn(
			move || {
				panic_handler.catch_panic(move || {
					Worker::work_loop(stealer, channel.clone(), wait, wait_mutex.clone(), deleting)
				}).unwrap()
			})
			.expect("Error creating worker thread"));
		worker
	}

	fn work_loop<Message>(stealer: chase_lev::Stealer<Work<Message>>,
						channel: IoChannel<Message>, wait: Arc<Condvar>,
						wait_mutex: Arc<Mutex<()>>,
						deleting: Arc<AtomicBool>)
						where Message: Send + Sync + Clone + 'static {
		loop {
			{
				let lock = wait_mutex.lock().unwrap();
				if deleting.load(AtomicOrdering::Acquire) {
					return;
				}
				let _ = wait.wait(lock).unwrap();
			}

			if deleting.load(AtomicOrdering::Acquire) {
				return;
			}
			while let chase_lev::Steal::Data(work) = stealer.steal() {
				Worker::do_work(work, channel.clone());
			}
		}
	}

	fn do_work<Message>(work: Work<Message>, channel: IoChannel<Message>) where Message: Send + Sync + Clone + 'static {
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
				work.handler.message(&IoContext::new(channel, work.handler_id), &message);
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
		let thread = mem::replace(&mut self.thread, None).unwrap();
		thread.join().ok();
		trace!(target: "shutdown", "[IoWorker] Closed");
	}
}
