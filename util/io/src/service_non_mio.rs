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

use std::sync::{Arc, Weak};
use std::thread;
use crossbeam::sync::chase_lev;
use slab::Slab;
use fnv::FnvHashMap;
use {IoError, IoHandler};
use parking_lot::{RwLock, Mutex};
use num_cpus;
use std::time::Duration;
use timer::{Timer, Guard as TimerGuard};
use time::Duration as TimeDuration;

/// Timer ID
pub type TimerToken = usize;
/// IO Handler ID
pub type HandlerId = usize;

/// Maximum number of tokens a handler can use
pub const TOKENS_PER_HANDLER: usize = 16384;
const MAX_HANDLERS: usize = 8;

/// IO access point. This is passed to all IO handlers and provides an interface to the IO subsystem.
pub struct IoContext<Message> where Message: Send + Sync + 'static {
	handler: HandlerId,
	shared: Arc<Shared<Message>>,
}

impl<Message> IoContext<Message> where Message: Send + Sync + 'static {
	/// Register a new recurring IO timer. 'IoHandler::timeout' will be called with the token.
	pub fn register_timer(&self, token: TimerToken, delay: Duration) -> Result<(), IoError> {
		let channel = self.channel();

		let msg = WorkTask::TimerTrigger {
			handler_id: self.handler,
			token: token,
		};

		let delay = TimeDuration::from_std(delay)
			.map_err(|e| ::std::io::Error::new(::std::io::ErrorKind::Other, e))?;
		let guard = self.shared.timer.lock().schedule_repeating(delay, move || {
			channel.send_raw(msg.clone());
		});

		self.shared.timers.lock().insert(token, guard);

		Ok(())
	}

	/// Register a new IO timer once. 'IoHandler::timeout' will be called with the token.
	pub fn register_timer_once(&self, token: TimerToken, delay: Duration) -> Result<(), IoError> {
		let channel = self.channel();

		let msg = WorkTask::TimerTrigger {
			handler_id: self.handler,
			token: token,
		};

		let delay = TimeDuration::from_std(delay)
			.map_err(|e| ::std::io::Error::new(::std::io::ErrorKind::Other, e))?;
		let guard = self.shared.timer.lock().schedule_with_delay(delay, move || {
			channel.send_raw(msg.clone());
		});

		self.shared.timers.lock().insert(token, guard);

		Ok(())
	}

	/// Delete a timer.
	pub fn clear_timer(&self, token: TimerToken) -> Result<(), IoError> {
		self.shared.timers.lock().remove(&token);
		Ok(())
	}

	/// Broadcast a message to other IO clients
	pub fn message(&self, message: Message) -> Result<(), IoError> {
		if let Some(ref channel) = *self.shared.channel.lock() {
			channel.push(WorkTask::UserMessage(Arc::new(message)));
		}
		for thread in self.shared.threads.read().iter() {
			thread.unpark();
		}

		Ok(())
	}

	/// Get message channel
	pub fn channel(&self) -> IoChannel<Message> {
		IoChannel { shared: Arc::downgrade(&self.shared) }
	}

	/// Unregister current IO handler.
	pub fn unregister_handler(&self) -> Result<(), IoError> {
		self.shared.handlers.write().remove(self.handler);
		Ok(())
	}
}

/// Allows sending messages into the event loop. All the IO handlers will get the message
/// in the `message` callback.
pub struct IoChannel<Message> where Message: Send + Sync + 'static {
	shared: Weak<Shared<Message>>,
}

impl<Message> Clone for IoChannel<Message> where Message: Send + Sync + 'static {
	fn clone(&self) -> IoChannel<Message> {
		IoChannel {
			shared: self.shared.clone(),
		}
	}
}

impl<Message> IoChannel<Message> where Message: Send + Sync + 'static {
	/// Send a message through the channel
	pub fn send(&self, message: Message) -> Result<(), IoError> {
		if let Some(shared) = self.shared.upgrade() {
			match *shared.channel.lock() {
				Some(ref channel) => channel.push(WorkTask::UserMessage(Arc::new(message))),
				None => self.send_sync(message)?
			};

			for thread in shared.threads.read().iter() {
				thread.unpark();
			}
		}

		Ok(())
	}

	/// Send a message through the channel and handle it synchronously
	pub fn send_sync(&self, message: Message) -> Result<(), IoError> {
		if let Some(shared) = self.shared.upgrade() {
			for id in 0 .. MAX_HANDLERS {
				if let Some(h) = shared.handlers.read().get(id) {
					let handler = h.clone();
					let ctxt = IoContext { handler: id, shared: shared.clone() };
					handler.message(&ctxt, &message);
				}
			}
		}

		Ok(())
	}

	// Send low level io message
	fn send_raw(&self, message: WorkTask<Message>) {
		if let Some(shared) = self.shared.upgrade() {
			if let Some(ref channel) = *shared.channel.lock() {
				channel.push(message);
			}

			for thread in shared.threads.read().iter() {
				thread.unpark();
			}
		}
	}

	/// Create a new channel disconnected from an event loop.
	pub fn disconnected() -> IoChannel<Message> {
		IoChannel {
			shared: Weak::default(),
		}
	}
}

/// General IO Service. Starts an event loop and dispatches IO requests.
/// 'Message' is a notification message type
pub struct IoService<Message> where Message: Send + Sync + 'static {
	thread_joins: Mutex<Vec<thread::JoinHandle<()>>>,
	shared: Arc<Shared<Message>>,
}

// Struct shared throughout the whole implementation.
struct Shared<Message> where Message: Send + Sync + 'static {
	// All the I/O handlers that have been registered.
	handlers: RwLock<Slab<Arc<IoHandler<Message>>>>,
	// All the background threads, so that we can unpark them.
	threads: RwLock<Vec<thread::Thread>>,
	// Used to create timeouts.
	timer: Mutex<Timer>,
	// List of created timers. We need to keep them in a data struct so that we can cancel them if
	// necessary.
	timers: Mutex<FnvHashMap<TimerToken, TimerGuard>>,
	// Channel used to send work to the worker threads.
	channel: Mutex<Option<chase_lev::Worker<WorkTask<Message>>>>,
}

// Messages used to communicate with the event loop from other threads.
enum WorkTask<Message> where Message: Send + Sized {
	Shutdown,
	TimerTrigger {
		handler_id: HandlerId,
		token: TimerToken,
	},
	UserMessage(Arc<Message>)
}

impl<Message> Clone for WorkTask<Message> where Message: Send + Sized {
	fn clone(&self) -> WorkTask<Message> {
		match *self {
			WorkTask::Shutdown => WorkTask::Shutdown,
			WorkTask::TimerTrigger { handler_id, token } => WorkTask::TimerTrigger { handler_id, token },
			WorkTask::UserMessage(ref msg) => WorkTask::UserMessage(msg.clone()),
		}
	}
}

impl<Message> IoService<Message> where Message: Send + Sync + 'static {
	/// Starts IO event loop
	pub fn start() -> Result<IoService<Message>, IoError> {
		let (tx, rx) = chase_lev::deque();

		let shared = Arc::new(Shared {
			handlers: RwLock::new(Slab::with_capacity(MAX_HANDLERS)),
			threads: RwLock::new(Vec::new()),
			timer: Mutex::new(Timer::new()),
			timers: Mutex::new(FnvHashMap::default()),
			channel: Mutex::new(Some(tx)),
		});

		let thread_joins = (0 .. num_cpus::get()).map(|_| {
			let rx = rx.clone();
			let shared = shared.clone();
			thread::spawn(move || {
				do_work(&shared, rx)
			})
		}).collect::<Vec<_>>();

		*shared.threads.write() = thread_joins.iter().map(|t| t.thread().clone()).collect();

		Ok(IoService {
			thread_joins: Mutex::new(thread_joins),
			shared,
		})
	}

	/// Stops the IO service.
	pub fn stop(&self) {
		trace!(target: "shutdown", "[IoService] Closing...");
		// Clear handlers so that shared pointers are not stuck on stack
		// in Channel::send_sync
		self.shared.handlers.write().clear();
		let channel = self.shared.channel.lock().take();
		let mut thread_joins = self.thread_joins.lock();
		if let Some(channel) = channel {
			for _ in 0 .. thread_joins.len() {
				channel.push(WorkTask::Shutdown);
			}
		}
		for thread in thread_joins.drain(..) {
			thread.thread().unpark();
			thread.join().unwrap_or_else(|e| {
				debug!(target: "shutdown", "Error joining IO service worker thread: {:?}", e);
			});
		}
		trace!(target: "shutdown", "[IoService] Closed.");
	}

	/// Register an IO handler with the event loop.
	pub fn register_handler(&self, handler: Arc<IoHandler<Message>+Send>) -> Result<(), IoError> {
		let id = self.shared.handlers.write().insert(handler.clone());
		assert!(id <= MAX_HANDLERS, "Too many handlers registered");
		let ctxt = IoContext { handler: id, shared: self.shared.clone() };
		handler.initialize(&ctxt);
		Ok(())
	}

	/// Send a message over the network. Normaly `HostIo::send` should be used. This can be used from non-io threads.
	pub fn send_message(&self, message: Message) -> Result<(), IoError> {
		if let Some(ref channel) = *self.shared.channel.lock() {
			channel.push(WorkTask::UserMessage(Arc::new(message)));
		}
		for thread in self.shared.threads.read().iter() {
			thread.unpark();
		}
		Ok(())
	}

	/// Create a new message channel
	#[inline]
	pub fn channel(&self) -> IoChannel<Message> {
		IoChannel {
			shared: Arc::downgrade(&self.shared)
		}
	}
}

impl<Message> Drop for IoService<Message> where Message: Send + Sync {
	fn drop(&mut self) {
		self.stop()
	}
}

fn do_work<Message>(shared: &Arc<Shared<Message>>, rx: chase_lev::Stealer<WorkTask<Message>>)
	where Message: Send + Sync + 'static 
{
	loop {
		match rx.steal() {
			chase_lev::Steal::Abort => continue,
			chase_lev::Steal::Empty => thread::park(),
			chase_lev::Steal::Data(WorkTask::Shutdown) => break,
			chase_lev::Steal::Data(WorkTask::UserMessage(message)) => {
				for id in 0 .. MAX_HANDLERS {
					if let Some(handler) = shared.handlers.read().get(id) {
						let ctxt = IoContext { handler: id, shared: shared.clone() };
						handler.message(&ctxt, &message);
					}
				}
			},
			chase_lev::Steal::Data(WorkTask::TimerTrigger { handler_id, token }) => {
				if let Some(handler) = shared.handlers.read().get(handler_id) {
					let ctxt = IoContext { handler: handler_id, shared: shared.clone() };
					handler.timeout(&ctxt, token);
				}
			},
		}
	}
}
