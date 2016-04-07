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
use std::thread::{self, JoinHandle};
use std::collections::HashMap;
use mio::*;
use hash::*;
use rlp::*;
use error::*;
use io::{IoError, IoHandler};
use arrayvec::*;
use crossbeam::sync::chase_lev;
use io::worker::{Worker, Work, WorkType};
use panics::*;

/// Timer ID
pub type TimerToken = usize;
/// Timer ID
pub type StreamToken = usize;
/// IO Hadndler ID
pub type HandlerId = usize;

/// Maximum number of tokens a handler can use
pub const TOKENS_PER_HANDLER: usize = 16384;

/// Messages used to communicate with the event loop from other threads.
#[derive(Clone)]
pub enum IoMessage<Message> where Message: Send + Clone + Sized {
	/// Shutdown the event loop
	Shutdown,
	/// Register a new protocol handler.
	AddHandler {
		handler: Arc<IoHandler<Message>+Send>,
	},
	AddTimer {
		handler_id: HandlerId,
		token: TimerToken,
		delay: u64,
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
pub struct IoContext<Message> where Message: Send + Clone + 'static {
	channel: IoChannel<Message>,
	handler: HandlerId,
}

impl<Message> IoContext<Message> where Message: Send + Clone + 'static {
	/// Create a new IO access point. Takes references to all the data that can be updated within the IO handler.
	pub fn new(channel: IoChannel<Message>, handler: HandlerId) -> IoContext<Message> {
		IoContext {
			handler: handler,
			channel: channel,
		}
	}

	/// Register a new IO timer. 'IoHandler::timeout' will be called with the token.
	pub fn register_timer(&self, token: TimerToken, ms: u64) -> Result<(), UtilError> {
		try!(self.channel.send_io(IoMessage::AddTimer {
			token: token,
			delay: ms,
			handler_id: self.handler,
		}));
		Ok(())
	}

	/// Delete a timer.
	pub fn clear_timer(&self, token: TimerToken) -> Result<(), UtilError> {
		try!(self.channel.send_io(IoMessage::RemoveTimer {
			token: token,
			handler_id: self.handler,
		}));
		Ok(())
	}

	/// Register a new IO stream.
	pub fn register_stream(&self, token: StreamToken) -> Result<(), UtilError> {
		try!(self.channel.send_io(IoMessage::RegisterStream {
			token: token,
			handler_id: self.handler,
		}));
		Ok(())
	}

	/// Deregister an IO stream.
	pub fn deregister_stream(&self, token: StreamToken) -> Result<(), UtilError> {
		try!(self.channel.send_io(IoMessage::DeregisterStream {
			token: token,
			handler_id: self.handler,
		}));
		Ok(())
	}

	/// Reregister an IO stream.
	pub fn update_registration(&self, token: StreamToken) -> Result<(), UtilError> {
		try!(self.channel.send_io(IoMessage::UpdateStreamRegistration {
			token: token,
			handler_id: self.handler,
		}));
		Ok(())
	}

	/// Broadcast a message to other IO clients
	pub fn message(&self, message: Message) {
		self.channel.send(message).expect("Error seding message");
	}

	/// Get message channel
	pub fn channel(&self) -> IoChannel<Message> {
		self.channel.clone()
	}
}

#[derive(Clone)]
struct UserTimer {
	delay: u64,
	timeout: Timeout,
}

/// Root IO handler. Manages user handlers, messages and IO timers.
pub struct IoManager<Message> where Message: Send + Sync {
	timers: Arc<RwLock<HashMap<HandlerId, UserTimer>>>,
	handlers: Vec<Arc<IoHandler<Message>>>,
	workers: Vec<Worker>,
	worker_channel: chase_lev::Worker<Work<Message>>,
	work_ready: Arc<Condvar>,
}

impl<Message> IoManager<Message> where Message: Send + Sync + Clone + 'static {
	/// Creates a new instance and registers it with the event loop.
	pub fn start(panic_handler: Arc<PanicHandler>, event_loop: &mut EventLoop<IoManager<Message>>) -> Result<(), UtilError> {
		let (worker, stealer) = chase_lev::deque();
		let num_workers = 4;
		let work_ready_mutex =  Arc::new(Mutex::new(()));
		let work_ready = Arc::new(Condvar::new());
		let workers = (0..num_workers).map(|i|
			Worker::new(
				i,
				stealer.clone(),
				IoChannel::new(event_loop.channel()),
				work_ready.clone(),
				work_ready_mutex.clone(),
				panic_handler.clone()
			)
		).collect();

		let mut io = IoManager {
			timers: Arc::new(RwLock::new(HashMap::new())),
			handlers: Vec::new(),
			worker_channel: worker,
			workers: workers,
			work_ready: work_ready,
		};
		try!(event_loop.run(&mut io));
		Ok(())
	}
}

impl<Message> Handler for IoManager<Message> where Message: Send + Clone + Sync + 'static {
	type Timeout = Token;
	type Message = IoMessage<Message>;

	fn ready(&mut self, _event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
		let handler_index  = token.as_usize() / TOKENS_PER_HANDLER;
		let token_id  = token.as_usize() % TOKENS_PER_HANDLER;
		if handler_index >= self.handlers.len() {
			panic!("Unexpected stream token: {}", token.as_usize());
		}
		let handler = self.handlers[handler_index].clone();

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

	fn timeout(&mut self, event_loop: &mut EventLoop<Self>, token: Token) {
		let handler_index  = token.as_usize()  / TOKENS_PER_HANDLER;
		let token_id  = token.as_usize()  % TOKENS_PER_HANDLER;
		if handler_index >= self.handlers.len() {
			panic!("Unexpected timer token: {}", token.as_usize());
		}
		if let Some(timer) = self.timers.read().unwrap().get(&token.as_usize()) {
			event_loop.timeout_ms(token, timer.delay).expect("Error re-registering user timer");
			let handler = self.handlers[handler_index].clone();
			self.worker_channel.push(Work { work_type: WorkType::Timeout, token: token_id, handler: handler, handler_id: handler_index });
			self.work_ready.notify_all();
		}
	}

	fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
		match msg {
			IoMessage::Shutdown => {
				self.workers.clear();
				event_loop.shutdown();
			},
			IoMessage::AddHandler { handler } => {
				let handler_id = {
					self.handlers.push(handler.clone());
					self.handlers.len() - 1
				};
				handler.initialize(&IoContext::new(IoChannel::new(event_loop.channel()), handler_id));
			},
			IoMessage::AddTimer { handler_id, token, delay } => {
				let timer_id = token + handler_id * TOKENS_PER_HANDLER;
				let timeout = event_loop.timeout_ms(Token(timer_id), delay).expect("Error registering user timer");
				self.timers.write().unwrap().insert(timer_id, UserTimer { delay: delay, timeout: timeout });
			},
			IoMessage::RemoveTimer { handler_id, token } => {
				let timer_id = token + handler_id * TOKENS_PER_HANDLER;
				if let Some(timer) = self.timers.write().unwrap().remove(&timer_id) {
					event_loop.clear_timeout(timer.timeout);
				}
			},
			IoMessage::RegisterStream { handler_id, token } => {
				let handler = self.handlers.get(handler_id).expect("Unknown handler id").clone();
				handler.register_stream(token, Token(token + handler_id * TOKENS_PER_HANDLER), event_loop);
			},
			IoMessage::DeregisterStream { handler_id, token } => {
				let handler = self.handlers.get(handler_id).expect("Unknown handler id").clone();
				handler.deregister_stream(token, event_loop);
				// unregister a timer associated with the token (if any)
				let timer_id = token + handler_id * TOKENS_PER_HANDLER;
				if let Some(timer) = self.timers.write().unwrap().remove(&timer_id) {
					event_loop.clear_timeout(timer.timeout);
				}
			},
			IoMessage::UpdateStreamRegistration { handler_id, token } => {
				let handler = self.handlers.get(handler_id).expect("Unknown handler id").clone();
				handler.update_stream(token, Token(token + handler_id * TOKENS_PER_HANDLER), event_loop);
			},
			IoMessage::UserMessage(data) => {
				for n in 0 .. self.handlers.len() {
					let handler = self.handlers[n].clone();
					self.worker_channel.push(Work { work_type: WorkType::Message(data.clone()), token: 0, handler: handler, handler_id: n });
				}
				self.work_ready.notify_all();
			}
		}
	}
}

/// Allows sending messages into the event loop. All the IO handlers will get the message
/// in the `message` callback.
pub struct IoChannel<Message> where Message: Send + Clone{
	channel: Option<Sender<IoMessage<Message>>>
}

impl<Message> Clone for IoChannel<Message> where Message: Send + Clone {
	fn clone(&self) -> IoChannel<Message> {
		IoChannel {
			channel: self.channel.clone()
		}
	}
}

impl<Message> IoChannel<Message> where Message: Send + Clone {
	/// Send a msessage through the channel
	pub fn send(&self, message: Message) -> Result<(), IoError> {
		if let Some(ref channel) = self.channel {
			try!(channel.send(IoMessage::UserMessage(message)));
		}
		Ok(())
	}

	/// Send low level io message
	pub fn send_io(&self, message: IoMessage<Message>) -> Result<(), IoError> {
		if let Some(ref channel) = self.channel {
			try!(channel.send(message))
		}
		Ok(())
	}
	/// Create a new channel to connected to event loop.
	pub fn disconnected() -> IoChannel<Message> {
		IoChannel { channel: None }
	}

	fn new(channel: Sender<IoMessage<Message>>) -> IoChannel<Message> {
		IoChannel { channel: Some(channel) }
	}
}

/// General IO Service. Starts an event loop and dispatches IO requests.
/// 'Message' is a notification message type
pub struct IoService<Message> where Message: Send + Sync + Clone + 'static {
	panic_handler: Arc<PanicHandler>,
	thread: Option<JoinHandle<()>>,
	host_channel: Sender<IoMessage<Message>>,
}

impl<Message> MayPanic for IoService<Message> where Message: Send + Sync + Clone + 'static {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}

impl<Message> IoService<Message> where Message: Send + Sync + Clone + 'static {
	/// Starts IO event loop
	pub fn start() -> Result<IoService<Message>, UtilError> {
		let panic_handler = PanicHandler::new_in_arc();
		let mut event_loop = EventLoop::new().unwrap();
        let channel = event_loop.channel();
		let panic = panic_handler.clone();
		let thread = thread::spawn(move || {
			let p = panic.clone();
			panic.catch_panic(move || {
				IoManager::<Message>::start(p, &mut event_loop).unwrap();
			}).unwrap()
		});
		Ok(IoService {
			panic_handler: panic_handler,
			thread: Some(thread),
			host_channel: channel
		})
	}

	/// Regiter a IO hadnler with the event loop.
	pub fn register_handler(&mut self, handler: Arc<IoHandler<Message>+Send>) -> Result<(), IoError> {
		try!(self.host_channel.send(IoMessage::AddHandler {
			handler: handler,
		}));
		Ok(())
	}

	/// Send a message over the network. Normaly `HostIo::send` should be used. This can be used from non-io threads.
	pub fn send_message(&mut self, message: Message) -> Result<(), IoError> {
		try!(self.host_channel.send(IoMessage::UserMessage(message)));
		Ok(())
	}

	/// Create a new message channel
	pub fn channel(&mut self) -> IoChannel<Message> {
		IoChannel { channel: Some(self.host_channel.clone()) }
	}
}

impl<Message> Drop for IoService<Message> where Message: Send + Sync + Clone {
	fn drop(&mut self) {
		trace!(target: "shutdown", "[IoService] Closing...");
		self.host_channel.send(IoMessage::Shutdown).unwrap();
		self.thread.take().unwrap().join().ok();
		trace!(target: "shutdown", "[IoService] Closed.");
	}
}

