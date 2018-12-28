// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! General IO module.
//!
//! Example usage for creating a network service and adding an IO handler:
//!
//! ```rust
//! extern crate ethcore_io;
//! use ethcore_io::*;
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! struct MyHandler;
//!
//! #[derive(Clone)]
//! struct MyMessage {
//! 	data: u32
//! }
//!
//! impl IoHandler<MyMessage> for MyHandler {
//! 	fn initialize(&self, io: &IoContext<MyMessage>) {
//!			io.register_timer(0, Duration::from_secs(1)).unwrap();
//!		}
//!
//!		fn timeout(&self, _io: &IoContext<MyMessage>, timer: TimerToken) {
//!			println!("Timeout {}", timer);
//!		}
//!
//!		fn message(&self, _io: &IoContext<MyMessage>, message: &MyMessage) {
//!			println!("Message {}", message.data);
//!		}
//! }
//!
//! fn main () {
//! 	let mut service = IoService::<MyMessage>::start().expect("Error creating network service");
//! 	service.register_handler(Arc::new(MyHandler)).unwrap();
//!
//! 	// Wait for quit condition
//! 	// ...
//! 	// Drop the service
//! }
//! ```
//!
//! # Mio vs non-mio
//!
//! This library has two modes: mio and not mio. The `mio` feature can be activated or deactivated
//! when compiling or depending on the library.
//!
//! Without mio, only timers and message-passing are available. With mio, you can also use
//! low-level sockets provided by mio.
//!
//! The non-mio mode exists because the `mio` library doesn't compile on platforms such as
//! emscripten.

//TODO: use Poll from mio
#![allow(deprecated)]

#[cfg(feature = "mio")]
extern crate mio;
#[macro_use]
extern crate log as rlog;
extern crate slab;
extern crate crossbeam_deque as deque;
extern crate parking_lot;
extern crate num_cpus;
extern crate timer;
extern crate fnv;
extern crate time;
extern crate tokio;
extern crate futures;

#[cfg(feature = "mio")]
mod service_mio;
#[cfg(not(feature = "mio"))]
mod service_non_mio;
#[cfg(feature = "mio")]
mod worker;

use std::cell::Cell;
use std::{fmt, error};
#[cfg(feature = "mio")]
use mio::deprecated::{EventLoop, NotifyError};
#[cfg(feature = "mio")]
use mio::Token;

thread_local! {
	/// Stack size
	/// Should be modified if it is changed in Rust since it is no way
	/// to know or get it
	pub static LOCAL_STACK_SIZE: Cell<usize> = Cell::new(::std::env::var("RUST_MIN_STACK").ok().and_then(|s| s.parse().ok()).unwrap_or(2 * 1024 * 1024));
}

#[derive(Debug)]
/// IO Error
pub enum IoError {
	/// Low level error from mio crate
	#[cfg(feature = "mio")]
	Mio(::std::io::Error),
	/// Error concerning the Rust standard library's IO subsystem.
	StdIo(::std::io::Error),
}

impl fmt::Display for IoError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// just defer to the std implementation for now.
		// we can refine the formatting when more variants are added.
		match *self {
			#[cfg(feature = "mio")]
			IoError::Mio(ref std_err) => std_err.fmt(f),
			IoError::StdIo(ref std_err) => std_err.fmt(f),
		}
	}
}

impl error::Error for IoError {
	fn description(&self) -> &str {
		"IO error"
	}
}

impl From<::std::io::Error> for IoError {
	fn from(err: ::std::io::Error) -> IoError {
		IoError::StdIo(err)
	}
}

#[cfg(feature = "mio")]
impl<Message> From<NotifyError<service_mio::IoMessage<Message>>> for IoError where Message: Send {
	fn from(_err: NotifyError<service_mio::IoMessage<Message>>) -> IoError {
		IoError::Mio(::std::io::Error::new(::std::io::ErrorKind::ConnectionAborted, "Network IO notification error"))
	}
}

/// Generic IO handler.
/// All the handler function are called from within IO event loop.
/// `Message` type is used as notification data
pub trait IoHandler<Message>: Send + Sync where Message: Send + Sync + 'static {
	/// Initialize the handler
	fn initialize(&self, _io: &IoContext<Message>) {}
	/// Timer function called after a timeout created with `HandlerIo::timeout`.
	fn timeout(&self, _io: &IoContext<Message>, _timer: TimerToken) {}
	/// Called when a broadcasted message is received. The message can only be sent from a different IO handler.
	fn message(&self, _io: &IoContext<Message>, _message: &Message) {}
	/// Called when an IO stream gets closed
	#[cfg(feature = "mio")]
	fn stream_hup(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
	/// Called when an IO stream can be read from
	#[cfg(feature = "mio")]
	fn stream_readable(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
	/// Called when an IO stream can be written to
	#[cfg(feature = "mio")]
	fn stream_writable(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
	/// Register a new stream with the event loop
	#[cfg(feature = "mio")]
	fn register_stream(&self, _stream: StreamToken, _reg: Token, _event_loop: &mut EventLoop<IoManager<Message>>) {}
	/// Re-register a stream with the event loop
	#[cfg(feature = "mio")]
	fn update_stream(&self, _stream: StreamToken, _reg: Token, _event_loop: &mut EventLoop<IoManager<Message>>) {}
	/// Deregister a stream. Called whenstream is removed from event loop
	#[cfg(feature = "mio")]
	fn deregister_stream(&self, _stream: StreamToken, _event_loop: &mut EventLoop<IoManager<Message>>) {}
}

#[cfg(feature = "mio")]
pub use service_mio::{TimerToken, StreamToken, IoContext, IoService, IoChannel, IoManager, TOKENS_PER_HANDLER};
#[cfg(not(feature = "mio"))]
pub use service_non_mio::{TimerToken, IoContext, IoService, IoChannel, TOKENS_PER_HANDLER};

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::sync::atomic;
	use std::thread;
	use std::time::Duration;
	use super::*;

	// Mio's behaviour is too unstable for this test. Sometimes we have to wait a few milliseconds,
	// sometimes more than 5 seconds for the message to arrive.
	// Therefore we ignore this test in order to not have spurious failure when running continuous
	// integration.
	#[test]
	#[cfg_attr(feature = "mio", ignore)]
	fn send_message_to_handler() {
		struct MyHandler(atomic::AtomicBool);

		#[derive(Clone)]
		struct MyMessage {
			data: u32
		}

		impl IoHandler<MyMessage> for MyHandler {
			fn message(&self, _io: &IoContext<MyMessage>, message: &MyMessage) {
				assert_eq!(message.data, 5);
				self.0.store(true, atomic::Ordering::SeqCst);
			}
		}

		let handler = Arc::new(MyHandler(atomic::AtomicBool::new(false)));

		let service = IoService::<MyMessage>::start().expect("Error creating network service");
		service.register_handler(handler.clone()).unwrap();

		service.send_message(MyMessage { data: 5 }).unwrap();

		thread::sleep(Duration::from_secs(1));
		assert!(handler.0.load(atomic::Ordering::SeqCst));
	}

	#[test]
	fn timeout_working() {
		struct MyHandler(atomic::AtomicBool);

		#[derive(Clone)]
		struct MyMessage {
			data: u32
		}

		impl IoHandler<MyMessage> for MyHandler {
			fn initialize(&self, io: &IoContext<MyMessage>) {
				io.register_timer_once(1234, Duration::from_millis(500)).unwrap();
			}

			fn timeout(&self, _io: &IoContext<MyMessage>, timer: TimerToken) {
				assert_eq!(timer, 1234);
				assert!(!self.0.swap(true, atomic::Ordering::SeqCst));
			}
		}

		let handler = Arc::new(MyHandler(atomic::AtomicBool::new(false)));

		let service = IoService::<MyMessage>::start().expect("Error creating network service");
		service.register_handler(handler.clone()).unwrap();

		thread::sleep(Duration::from_secs(2));
		assert!(handler.0.load(atomic::Ordering::SeqCst));
	}

	#[test]
	fn multi_timeout_working() {
		struct MyHandler(atomic::AtomicUsize);

		#[derive(Clone)]
		struct MyMessage {
			data: u32
		}

		impl IoHandler<MyMessage> for MyHandler {
			fn initialize(&self, io: &IoContext<MyMessage>) {
				io.register_timer(1234, Duration::from_millis(500)).unwrap();
			}

			fn timeout(&self, _io: &IoContext<MyMessage>, timer: TimerToken) {
				assert_eq!(timer, 1234);
				self.0.fetch_add(1, atomic::Ordering::SeqCst);
			}
		}

		let handler = Arc::new(MyHandler(atomic::AtomicUsize::new(0)));

		let service = IoService::<MyMessage>::start().expect("Error creating network service");
		service.register_handler(handler.clone()).unwrap();

		thread::sleep(Duration::from_secs(2));
		assert!(handler.0.load(atomic::Ordering::SeqCst) >= 2);
	}
}
