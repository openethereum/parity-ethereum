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
use std::thread::{self, JoinHandle};
use std::collections::HashMap;
use mio::*;
use mio::timer::{Timeout};
use mio::deprecated::{EventLoop, Handler, Sender, EventLoopBuilder};
use crossbeam::sync::chase_lev;
use slab::Slab;
use {IoError, IoHandler};
use worker::{Worker, Work, WorkType};
use parking_lot::{RwLock, Mutex};
use std::sync::{Condvar as SCondvar, Mutex as SMutex};
use std::time::Duration;

/// Timer ID
pub type TimerToken = usize;
/// Timer ID
pub type StreamToken = usize;
/// IO Hadndler ID
pub type HandlerId = usize;

/// Maximum number of tokens a handler can use
pub const TOKENS_PER_HANDLER: usize = 16384;
const MAX_HANDLERS: usize = 8;

/// Messages used to communicate with the event loop from other threads.
#[derive(Clone)]
pub enum IoMessage<Message> where Message: Send + Clone + Sized {
	/// Shutdown the event loop
	Shutdown,
	/// Register a new protocol handler.
	AddHandler {
		handler: Arc<IoHandler<Message>+Send>,
	},
	RemoveHandler {
		handler_id: HandlerId,
	},
	AddTimer {
		handler_id: HandlerId,
		token: TimerToken,
		delay: Duration,
		once: bool,
	},
	RemoveTimer {
		handler_id: HandlerId,
		token: TimerToken,
	},
	RegisterStream {
		handler_id: HandlerId,
		token: StreamToken,
	},
	DeregisterStream {
		handler_id: HandlerId,
		token: StreamToken,
	},
	UpdateStreamRegistration {
		handler_id: HandlerId,
		token: StreamToken,
	},
	/// Broadcast a message across all protocol handlers.
	UserMessage(Message)
}

/// IO access point. This is passed to all IO handlers and provides an interface to the IO subsystem.
pub struct IoContext<Message> where Message: Send + Clone + Sync + 'static {
	channel: IoChannel<Message>,
	handler: HandlerId,
}

impl<Message> IoContext<Message> where Message: Send + Clone + Sync + 'static {
	/// Create a new IO access point. Takes references to all the data that can be updated within the IO handler.
	pub fn new(channel: IoChannel<Message>, handler: HandlerId) -> IoContext<Message> {
		IoContext {
			handler: handler,
			channel: channel,
		}
	}

	/// Register a new recurring IO timer. 'IoHandler::timeout' will be called with the token.
	pub fn register_timer(&self, token: TimerToken, delay: Duration) -> Result<(), IoError> {
		self.channel.send_io(IoMessage::AddTimer {
			token,
			delay,
			handler_id: self.handler,
			once: false,
		})?;
		Ok(())
	}

	/// Register a new IO timer once. 'IoHandler::timeout' will be called with the token.
	pub fn register_timer_once(&self, token: TimerToken, delay: Duration) -> Result<(), IoError> {
		self.channel.send_io(IoMessage::AddTimer {
			token,
			delay,
			handler_id: self.handler,
			once: true,
		})?;
		Ok(())
	}

	/// Delete a timer.
	pub fn clear_timer(&self, token: TimerToken) -> Result<(), IoError> {
		self.channel.send_io(IoMessage::RemoveTimer {
			token: token,
			handler_id: self.handler,
		})?;
		Ok(())
	}

	/// Register a new IO stream.
	pub fn register_stream(&self, token: StreamToken) -> Result<(), IoError> {
		self.channel.send_io(IoMessage::RegisterStream {
			token: token,
			handler_id: self.handler,
		})?;
		Ok(())
	}

	/// Deregister an IO stream.
	pub fn deregister_stream(&self, token: StreamToken) -> Result<(), IoError> {
		self.channel.send_io(IoMessage::DeregisterStream {
			token: token,
			handler_id: self.handler,
		})?;
		Ok(())
	}

	/// Reregister an IO stream.
	pub fn update_registration(&self, token: StreamToken) -> Result<(), IoError> {
		self.channel.send_io(IoMessage::UpdateStreamRegistration {
			token: token,
			handler_id: self.handler,
		})?;
		Ok(())
	}

	/// Broadcast a message to other IO clients
	pub fn message(&self, message: Message) -> Result<(), IoError> {
		self.channel.send(message)?;
		Ok(())
	}

	/// Get message channel
	pub fn channel(&self) -> IoChannel<Message> {
		self.channel.clone()
	}

	/// Unregister current IO handler.
	pub fn unregister_handler(&self) -> Result<(), IoError> {
		self.channel.send_io(IoMessage::RemoveHandler {
			handler_id: self.handler,
		})?;
		Ok(())
	}

}

#[derive(Clone)]
struct UserTimer {
	delay: Duration,
	timeout: Timeout,
	once: bool,
}

/// Root IO handler. Manages user handlers, messages and IO timers.
pub struct IoManager<Message> where Message: Send + Sync {
	timers: Arc<RwLock<HashMap<HandlerId, UserTimer>>>,
	handlers: Arc<RwLock<Slab<Arc<IoHandler<Message>>, HandlerId>>>,
	workers: Vec<Worker>,
	worker_channel: chase_lev::Worker<Work<Message>>,
	work_ready: Arc<SCondvar>,
}

impl<Message> IoManager<Message> where Message: Send + Sync + Clone + 'static {
	/// Creates a new instance and registers it with the event loop.
	pub fn start(
		event_loop: &mut EventLoop<IoManager<Message>>,
		handlers: Arc<RwLock<Slab<Arc<IoHandler<Message>>, HandlerId>>>
	) -> Result<(), IoError> {
		let (worker, stealer) = chase_lev::deque();
		let num_workers = 4;
		let work_ready_mutex =  Arc::new(SMutex::new(()));
		let work_ready = Arc::new(SCondvar::new());
		let workers = (0..num_workers).map(|i|
			Worker::new(
				i,
				stealer.clone(),
				IoChannel::new(event_loop.channel(), Arc::downgrade(&handlers)),
				work_ready.clone(),
				work_ready_mutex.clone(),
			)
		).collect();

		let mut io = IoManager {
			timers: Arc::new(RwLock::new(HashMap::new())),
			handlers: handlers,
			worker_channel: worker,
			workers: workers,
			work_ready: work_ready,
		};
		event_loop.run(&mut io)?;
		Ok(())
	}
}

impl<Message> Handler for IoManager<Message> where Message: Send + Clone + Sync + 'static {
	type Timeout = Token;
	type Message = IoMessage<Message>;

	fn ready(&mut self, _event_loop: &mut EventLoop<Self>, token: Token, events: Ready) {
		let handler_index  = token.0 / TOKENS_PER_HANDLER;
		let token_id  = token.0 % TOKENS_PER_HANDLER;
		if let Some(handler) = self.handlers.read().get(handler_index) {
			if events.is_hup() {
				self.worker_channel.push(Work { work_type: WorkType::Hup, token: token_id, handler: handler.clone(), handler_id: handler_index });
			}
			else {
				if events.is_readable() {
					self.worker_channel.push(Work { work_type: WorkType::Readable, token: token_id, handler: handler.clone(), handler_id: handler_index });
				}
				if events.is_writable() {
					self.worker_channel.push(Work { work_type: WorkType::Writable, token: token_id, handler: handler.clone(), handler_id: handler_index });
				}
			}
			self.work_ready.notify_all();
		}
	}

	fn timeout(&mut self, event_loop: &mut EventLoop<Self>, token: Token) {
		let handler_index  = token.0  / TOKENS_PER_HANDLER;
		let token_id  = token.0  % TOKENS_PER_HANDLER;
		if let Some(handler) = self.handlers.read().get(handler_index) {
			let maybe_timer = self.timers.read().get(&token.0).cloned();
			if let Some(timer) = maybe_timer {
				if timer.once {
					self.timers.write().remove(&token_id);
					event_loop.clear_timeout(&timer.timeout);
				} else {
					event_loop.timeout(token, timer.delay).expect("Error re-registering user timer");
				}
				self.worker_channel.push(Work { work_type: WorkType::Timeout, token: token_id, handler: handler.clone(), handler_id: handler_index });
				self.work_ready.notify_all();
			}
		}
	}

	fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
		match msg {
			IoMessage::Shutdown => {
				self.workers.clear();
				event_loop.shutdown();
			},
			IoMessage::AddHandler { handler } => {
				let handler_id = self.handlers.write().insert(handler.clone()).unwrap_or_else(|_| panic!("Too many handlers registered"));
				handler.initialize(&IoContext::new(IoChannel::new(event_loop.channel(), Arc::downgrade(&self.handlers)), handler_id));
			},
			IoMessage::RemoveHandler { handler_id } => {
				// TODO: flush event loop
				self.handlers.write().remove(handler_id);
				// unregister timers
				let mut timers = self.timers.write();
				let to_remove: Vec<_> = timers.keys().cloned().filter(|timer_id| timer_id / TOKENS_PER_HANDLER == handler_id).collect();
				for timer_id in to_remove {
					let timer = timers.remove(&timer_id).expect("to_remove only contains keys from timers; qed");
					event_loop.clear_timeout(&timer.timeout);
				}
			},
			IoMessage::AddTimer { handler_id, token, delay, once } => {
				let timer_id = token + handler_id * TOKENS_PER_HANDLER;
				let timeout = event_loop.timeout(Token(timer_id), delay).expect("Error registering user timer");
				self.timers.write().insert(timer_id, UserTimer { delay: delay, timeout: timeout, once: once });
			},
			IoMessage::RemoveTimer { handler_id, token } => {
				let timer_id = token + handler_id * TOKENS_PER_HANDLER;
				if let Some(timer) = self.timers.write().remove(&timer_id) {
					event_loop.clear_timeout(&timer.timeout);
				}
			},
			IoMessage::RegisterStream { handler_id, token } => {
				if let Some(handler) = self.handlers.read().get(handler_id) {
					handler.register_stream(token, Token(token + handler_id * TOKENS_PER_HANDLER), event_loop);
				}
			},
			IoMessage::DeregisterStream { handler_id, token } => {
				if let Some(handler) = self.handlers.read().get(handler_id) {
					handler.deregister_stream(token, event_loop);
					// unregister a timer associated with the token (if any)
					let timer_id = token + handler_id * TOKENS_PER_HANDLER;
					if let Some(timer) = self.timers.write().remove(&timer_id) {
						event_loop.clear_timeout(&timer.timeout);
					}
				}
			},
			IoMessage::UpdateStreamRegistration { handler_id, token } => {
				if let Some(handler) = self.handlers.read().get(handler_id) {
					handler.update_stream(token, Token(token + handler_id * TOKENS_PER_HANDLER), event_loop);
				}
			},
			IoMessage::UserMessage(data) => {
				//TODO: better way to iterate the slab
				for id in 0 .. MAX_HANDLERS {
					if let Some(h) = self.handlers.read().get(id) {
						let handler = h.clone();
						self.worker_channel.push(Work { work_type: WorkType::Message(data.clone()), token: 0, handler: handler, handler_id: id });
					}
				}
				self.work_ready.notify_all();
			}
		}
	}
}

#[derive(Clone)]
enum Handlers<Message> where Message: Send + Clone {
	SharedCollection(Weak<RwLock<Slab<Arc<IoHandler<Message>>, HandlerId>>>),
	Single(Weak<IoHandler<Message>>),
}

/// Allows sending messages into the event loop. All the IO handlers will get the message
/// in the `message` callback.
pub struct IoChannel<Message> where Message: Send + Clone{
	channel: Option<Sender<IoMessage<Message>>>,
	handlers: Handlers<Message>,

}

impl<Message> Clone for IoChannel<Message> where Message: Send + Clone + Sync + 'static {
	fn clone(&self) -> IoChannel<Message> {
		IoChannel {
			channel: self.channel.clone(),
			handlers: self.handlers.clone(),
		}
	}
}

impl<Message> IoChannel<Message> where Message: Send + Clone + Sync + 'static {
	/// Send a message through the channel
	pub fn send(&self, message: Message) -> Result<(), IoError> {
		match self.channel {
			Some(ref channel) => channel.send(IoMessage::UserMessage(message))?,
			None => self.send_sync(message)?
		}
		Ok(())
	}

	/// Send a message through the channel and handle it synchronously
	pub fn send_sync(&self, message: Message) -> Result<(), IoError> {
		match self.handlers {
			Handlers::SharedCollection(ref handlers) => {
				if let Some(handlers) = handlers.upgrade() {
					for id in 0 .. MAX_HANDLERS {
						if let Some(h) = handlers.read().get(id) {
							let handler = h.clone();
							handler.message(&IoContext::new(self.clone(), id), &message);
						}
					}
				}
			},
			Handlers::Single(ref handler) => {
				if let Some(handler) = handler.upgrade() {
					handler.message(&IoContext::new(self.clone(), 0), &message);
				}
			}
		}
		Ok(())
	}

	/// Send low level io message
	pub fn send_io(&self, message: IoMessage<Message>) -> Result<(), IoError> {
		if let Some(ref channel) = self.channel {
			channel.send(message)?
		}
		Ok(())
	}
	/// Create a new channel disconnected from an event loop.
	pub fn disconnected() -> IoChannel<Message> {
		IoChannel {
			channel: None,
			handlers: Handlers::SharedCollection(Weak::default()),
		}
	}

	/// Create a new synchronous channel to a given handler.
	pub fn to_handler(handler: Weak<IoHandler<Message>>) -> IoChannel<Message> {
		IoChannel {
			channel: None,
			handlers: Handlers::Single(handler),
		}
	}
	fn new(channel: Sender<IoMessage<Message>>, handlers: Weak<RwLock<Slab<Arc<IoHandler<Message>>, HandlerId>>>) -> IoChannel<Message> {
		IoChannel {
			channel: Some(channel),
			handlers: Handlers::SharedCollection(handlers),
		}
	}
}

/// General IO Service. Starts an event loop and dispatches IO requests.
/// 'Message' is a notification message type
pub struct IoService<Message> where Message: Send + Sync + Clone + 'static {
	thread: Mutex<Option<JoinHandle<()>>>,
	host_channel: Mutex<Sender<IoMessage<Message>>>,
	handlers: Arc<RwLock<Slab<Arc<IoHandler<Message>>, HandlerId>>>,
}

impl<Message> IoService<Message> where Message: Send + Sync + Clone + 'static {
	/// Starts IO event loop
	pub fn start() -> Result<IoService<Message>, IoError> {
		let mut config = EventLoopBuilder::new();
		config.messages_per_tick(1024);
		let mut event_loop = config.build().expect("Error creating event loop");
		let channel = event_loop.channel();
		let handlers = Arc::new(RwLock::new(Slab::new(MAX_HANDLERS)));
		let h = handlers.clone();
		let thread = thread::spawn(move || {
			IoManager::<Message>::start(&mut event_loop, h).expect("Error starting IO service");
		});
		Ok(IoService {
			thread: Mutex::new(Some(thread)),
			host_channel: Mutex::new(channel),
			handlers: handlers,
		})
	}

	pub fn stop(&self) {
		trace!(target: "shutdown", "[IoService] Closing...");
		// Clear handlers so that shared pointers are not stuck on stack
		// in Channel::send_sync
		self.handlers.write().clear();
		self.host_channel.lock().send(IoMessage::Shutdown).unwrap_or_else(|e| warn!("Error on IO service shutdown: {:?}", e));
		if let Some(thread) = self.thread.lock().take() {
			thread.join().unwrap_or_else(|e| {
				debug!(target: "shutdown", "Error joining IO service event loop thread: {:?}", e);
			});
		}
		trace!(target: "shutdown", "[IoService] Closed.");
	}

	/// Regiter an IO handler with the event loop.
	pub fn register_handler(&self, handler: Arc<IoHandler<Message>+Send>) -> Result<(), IoError> {
		self.host_channel.lock().send(IoMessage::AddHandler {
			handler: handler,
		})?;
		Ok(())
	}

	/// Send a message over the network. Normaly `HostIo::send` should be used. This can be used from non-io threads.
	pub fn send_message(&self, message: Message) -> Result<(), IoError> {
		self.host_channel.lock().send(IoMessage::UserMessage(message))?;
		Ok(())
	}

	/// Create a new message channel
	pub fn channel(&self) -> IoChannel<Message> {
		IoChannel::new(self.host_channel.lock().clone(), Arc::downgrade(&self.handlers))
	}
}

impl<Message> Drop for IoService<Message> where Message: Send + Sync + Clone {
	fn drop(&mut self) {
		self.stop()
	}
}

