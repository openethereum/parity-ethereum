use std::thread::{self, JoinHandle};
use mio::*;
use mio::util::{Slab};
use hash::*;
use rlp::*;
use error::*;
use io::{IoError, IoHandler};

pub type TimerToken = usize;
pub type StreamToken = usize;

// Tokens
const MAX_USER_TIMERS: usize = 32;
const USER_TIMER: usize = 0;
const LAST_USER_TIMER: usize = USER_TIMER + MAX_USER_TIMERS - 1;
pub const USER_TOKEN: usize = LAST_USER_TIMER + 1;

/// Messages used to communicate with the event loop from other threads.
pub enum IoMessage<M> where M: Send + Sized {
	/// Shutdown the event loop
	Shutdown,
	/// Register a new protocol handler.
	AddHandler {
		handler: Box<IoHandler<M>+Send>,
	},
	/// Broadcast a message across all protocol handlers.
	UserMessage(M)
}

/// IO access point. This is passed to all IO handlers and provides an interface to the IO subsystem.
pub struct IoContext<'s, M> where M: Send + 'static {
	timers: &'s mut Slab<UserTimer>,
	pub event_loop: &'s mut EventLoop<IoManager<M>>,
}

impl<'s, M> IoContext<'s, M> where M: Send + 'static {
	/// Create a new IO access point. Takes references to all the data that can be updated within the IO handler.
	fn new(event_loop: &'s mut EventLoop<IoManager<M>>, timers: &'s mut Slab<UserTimer>) -> IoContext<'s, M> {
		IoContext {
			event_loop: event_loop,
			timers: timers,
		}
	}

	/// Register a new IO timer. Returns a new timer token. 'IoHandler::timeout' will be called with the token.
	pub fn register_timer(&mut self, ms: u64) -> Result<TimerToken, UtilError>{
		match self.timers.insert(UserTimer {
			delay: ms,
		}) {
			Ok(token) => {
				self.event_loop.timeout_ms(token, ms).expect("Error registering user timer");
				Ok(token.as_usize())
			},
			_ => { panic!("Max timers reached") }
		}
	}

	/// Broadcast a message to other IO clients
	pub fn message(&mut self, message: M) {
		match self.event_loop.channel().send(IoMessage::UserMessage(message)) {
			Ok(_) => {}
			Err(e) => { panic!("Error sending io message {:?}", e); }
		}
	}
}

struct UserTimer {
	delay: u64,
}

/// Root IO handler. Manages user handlers, messages and IO timers.
pub struct IoManager<M> where M: Send {
	timers: Slab<UserTimer>,
	handlers: Vec<Box<IoHandler<M>>>,
}

impl<M> IoManager<M> where M: Send + 'static {
	/// Creates a new instance and registers it with the event loop.
	pub fn start(event_loop: &mut EventLoop<IoManager<M>>) -> Result<(), UtilError> {
		let mut io = IoManager {
			timers: Slab::new_starting_at(Token(USER_TIMER), MAX_USER_TIMERS),
			handlers: Vec::new(),
		};
		try!(event_loop.run(&mut io));
		Ok(())
	}
}

impl<M> Handler for IoManager<M> where M: Send + 'static {
	type Timeout = Token;
	type Message = IoMessage<M>;

	fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
		if events.is_hup() {
			for h in self.handlers.iter_mut() {
				h.stream_hup(IoContext::new(event_loop, &mut self.timers), token.as_usize());
			}
		}
		else if events.is_readable() {
			for h in self.handlers.iter_mut() {
				h.stream_readable(IoContext::new(event_loop, &mut self.timers), token.as_usize());
			}
		}
		else if events.is_writable() {
			for h in self.handlers.iter_mut() {
				h.stream_writable(IoContext::new(event_loop, &mut self.timers), token.as_usize());
			}
		}
	}

	fn timeout(&mut self, event_loop: &mut EventLoop<Self>, token: Token) {
		match token.as_usize() {
			USER_TIMER ... LAST_USER_TIMER => {
				let delay = {
					let timer = self.timers.get_mut(token).expect("Unknown user timer token");
					timer.delay
				};
				for h in self.handlers.iter_mut() {
					h.timeout(IoContext::new(event_loop, &mut self.timers), token.as_usize());
				}
				event_loop.timeout_ms(token, delay).expect("Error re-registering user timer");
			}
			_ => { // Just pass the event down. IoHandler is supposed to re-register it if required.
				for h in self.handlers.iter_mut() {
					h.timeout(IoContext::new(event_loop, &mut self.timers), token.as_usize());
				}
			}
		}
	}

	fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
		let mut m = msg;
		match m {
			IoMessage::Shutdown => event_loop.shutdown(),
			IoMessage::AddHandler {
				handler,
			} => {
				self.handlers.push(handler);
			},
			IoMessage::UserMessage(ref mut data) => {
				for h in self.handlers.iter_mut() {
					h.message(IoContext::new(event_loop, &mut self.timers), data);
				}
			}
		}
	}
}

/// General IO Service. Starts an event loop and dispatches IO requests.
/// 'M' is a notification message type
pub struct IoService<M> where M: Send + 'static {
	thread: Option<JoinHandle<()>>,
	host_channel: Sender<IoMessage<M>>
}

impl<M> IoService<M> where M: Send + 'static {
	/// Starts IO event loop
	pub fn start() -> Result<IoService<M>, UtilError> {
		let mut event_loop = EventLoop::new().unwrap();
        let channel = event_loop.channel();
		let thread = thread::spawn(move || {
			IoManager::<M>::start(&mut event_loop).unwrap(); //TODO:
		});
		Ok(IoService {
			thread: Some(thread),
			host_channel: channel
		})
	}

	/// Regiter a IO hadnler with the event loop.
	pub fn register_handler(&mut self, handler: Box<IoHandler<M>+Send>) -> Result<(), IoError> {
		try!(self.host_channel.send(IoMessage::AddHandler {
			handler: handler,
		}));
		Ok(())
	}

	/// Send a message over the network. Normaly `HostIo::send` should be used. This can be used from non-io threads.
	pub fn send_message(&mut self, message: M) -> Result<(), IoError> {
		try!(self.host_channel.send(IoMessage::UserMessage(message)));
		Ok(())
	}
}

impl<M> Drop for IoService<M> where M: Send {
	fn drop(&mut self) {
		self.host_channel.send(IoMessage::Shutdown).unwrap();
		self.thread.take().unwrap().join().unwrap();
	}
}

