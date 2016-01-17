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
//const USER_TOKEN: usize = LAST_USER_TIMER + 1;

/// Messages used to communicate with the event loop from other threads.
pub enum IoMessage<Message> where Message: Send + Sized {
	/// Shutdown the event loop
	Shutdown,
	/// Register a new protocol handler.
	AddHandler {
		handler: Box<IoHandler<Message>+Send>,
	},
	/// Broadcast a message across all protocol handlers.
	UserMessage(Message)
}

/// IO access point. This is passed to all IO handlers and provides an interface to the IO subsystem.
pub struct IoContext<'s, Message> where Message: Send + 'static {
	timers: &'s mut Slab<UserTimer>,
	/// Low leve MIO Event loop for custom handler registration.
	pub event_loop: &'s mut EventLoop<IoManager<Message>>,
}

impl<'s, Message> IoContext<'s, Message> where Message: Send + 'static {
	/// Create a new IO access point. Takes references to all the data that can be updated within the IO handler.
	fn new(event_loop: &'s mut EventLoop<IoManager<Message>>, timers: &'s mut Slab<UserTimer>) -> IoContext<'s, Message> {
		IoContext {
			event_loop: event_loop,
			timers: timers,
		}
	}

	/// Register a new IO timer. Returns a new timer token. 'IoHandler::timeout' will be called with the token.
	pub fn register_timer(&mut self, ms: u64) -> Result<TimerToken, UtilError> {
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
	pub fn message(&mut self, message: Message) {
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
pub struct IoManager<Message> where Message: Send {
	timers: Slab<UserTimer>,
	handlers: Vec<Box<IoHandler<Message>>>,
}

impl<Message> IoManager<Message> where Message: Send + 'static {
	/// Creates a new instance and registers it with the event loop.
	pub fn start(event_loop: &mut EventLoop<IoManager<Message>>) -> Result<(), UtilError> {
		let mut io = IoManager {
			timers: Slab::new_starting_at(Token(USER_TIMER), MAX_USER_TIMERS),
			handlers: Vec::new(),
		};
		try!(event_loop.run(&mut io));
		Ok(())
	}
}

impl<Message> Handler for IoManager<Message> where Message: Send + 'static {
	type Timeout = Token;
	type Message = IoMessage<Message>;

	fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
		if events.is_hup() {
			for h in self.handlers.iter_mut() {
				h.stream_hup(&mut IoContext::new(event_loop, &mut self.timers), token.as_usize());
			}
		}
		else if events.is_readable() {
			for h in self.handlers.iter_mut() {
				h.stream_readable(&mut IoContext::new(event_loop, &mut self.timers), token.as_usize());
			}
		}
		else if events.is_writable() {
			for h in self.handlers.iter_mut() {
				h.stream_writable(&mut IoContext::new(event_loop, &mut self.timers), token.as_usize());
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
					h.timeout(&mut IoContext::new(event_loop, &mut self.timers), token.as_usize());
				}
				event_loop.timeout_ms(token, delay).expect("Error re-registering user timer");
			}
			_ => { // Just pass the event down. IoHandler is supposed to re-register it if required.
				for h in self.handlers.iter_mut() {
					h.timeout(&mut IoContext::new(event_loop, &mut self.timers), token.as_usize());
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
				self.handlers.last_mut().unwrap().initialize(&mut IoContext::new(event_loop, &mut self.timers));
			},
			IoMessage::UserMessage(ref mut data) => {
				for h in self.handlers.iter_mut() {
					h.message(&mut IoContext::new(event_loop, &mut self.timers), data);
				}
			}
		}
	}
}

/// Allows sending messages into the event loop. All the IO handlers will get the message
/// in the `message` callback.
pub struct IoChannel<Message> where Message: Send {
	channel: Sender<IoMessage<Message>> 
}

impl<Message> IoChannel<Message> where Message: Send {
	pub fn send(&self, message: Message) -> Result<(), IoError> {
		try!(self.channel.send(IoMessage::UserMessage(message)));
		Ok(())
	}
}

/// General IO Service. Starts an event loop and dispatches IO requests.
/// 'Message' is a notification message type
pub struct IoService<Message> where Message: Send + 'static {
	thread: Option<JoinHandle<()>>,
	host_channel: Sender<IoMessage<Message>>
}

impl<Message> IoService<Message> where Message: Send + 'static {
	/// Starts IO event loop
	pub fn start() -> Result<IoService<Message>, UtilError> {
		let mut event_loop = EventLoop::new().unwrap();
        let channel = event_loop.channel();
		let thread = thread::spawn(move || {
			IoManager::<Message>::start(&mut event_loop).unwrap(); //TODO:
		});
		Ok(IoService {
			thread: Some(thread),
			host_channel: channel
		})
	}

	/// Regiter a IO hadnler with the event loop.
	pub fn register_handler(&mut self, handler: Box<IoHandler<Message>+Send>) -> Result<(), IoError> {
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
		IoChannel { channel: self.host_channel.clone() }
	}
}

impl<Message> Drop for IoService<Message> where Message: Send {
	fn drop(&mut self) {
		self.host_channel.send(IoMessage::Shutdown).unwrap();
		self.thread.take().unwrap().join().unwrap();
	}
}

