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

//! General IO module.
//!
//! Example usage for creating a network service and adding an IO handler:
//!
//! ```rust
//! extern crate ethcore_util;
//! use ethcore_util::*;
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
//!			io.register_timer(0, 1000).unwrap();
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
mod service;
mod worker;

use mio::{EventLoop, Token};
use std::fmt;

#[derive(Debug)]
/// IO Error
pub enum IoError {
	/// Low level error from mio crate
	Mio(::std::io::Error),
}

impl fmt::Display for IoError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// just defer to the std implementation for now.
		// we can refine the formatting when more variants are added.
		match *self {
			IoError::Mio(ref std_err) => std_err.fmt(f)
		}
	}
}

impl<Message> From<::mio::NotifyError<service::IoMessage<Message>>> for IoError where Message: Send + Clone {
	fn from(_err: ::mio::NotifyError<service::IoMessage<Message>>) -> IoError {
		IoError::Mio(::std::io::Error::new(::std::io::ErrorKind::ConnectionAborted, "Network IO notification error"))
	}
}

/// Generic IO handler.
/// All the handler function are called from within IO event loop.
/// `Message` type is used as notification data
pub trait IoHandler<Message>: Send + Sync where Message: Send + Sync + Clone + 'static {
	/// Initialize the handler
	fn initialize(&self, _io: &IoContext<Message>) {}
	/// Timer function called after a timeout created with `HandlerIo::timeout`.
	fn timeout(&self, _io: &IoContext<Message>, _timer: TimerToken) {}
	/// Called when a broadcasted message is received. The message can only be sent from a different IO handler.
	fn message(&self, _io: &IoContext<Message>, _message: &Message) {}
	/// Called when an IO stream gets closed
	fn stream_hup(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
	/// Called when an IO stream can be read from
	fn stream_readable(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
	/// Called when an IO stream can be written to
	fn stream_writable(&self, _io: &IoContext<Message>, _stream: StreamToken) {}
	/// Register a new stream with the event loop
	fn register_stream(&self, _stream: StreamToken, _reg: Token, _event_loop: &mut EventLoop<IoManager<Message>>) {}
	/// Re-register a stream with the event loop
	fn update_stream(&self, _stream: StreamToken, _reg: Token, _event_loop: &mut EventLoop<IoManager<Message>>) {}
	/// Deregister a stream. Called whenstream is removed from event loop
	fn deregister_stream(&self, _stream: StreamToken, _event_loop: &mut EventLoop<IoManager<Message>>) {}
}

pub use io::service::TimerToken;
pub use io::service::StreamToken;
pub use io::service::IoContext;
pub use io::service::IoService;
pub use io::service::IoChannel;
pub use io::service::IoManager;
pub use io::service::TOKENS_PER_HANDLER;

#[cfg(test)]
mod tests {

	use std::sync::Arc;
	use io::*;

	struct MyHandler;

	#[derive(Clone)]
	struct MyMessage {
		data: u32
	}

	impl IoHandler<MyMessage> for MyHandler {
		fn initialize(&self, io: &IoContext<MyMessage>) {
			io.register_timer(0, 1000).unwrap();
		}

		fn timeout(&self, _io: &IoContext<MyMessage>, timer: TimerToken) {
			println!("Timeout {}", timer);
		}

		fn message(&self, _io: &IoContext<MyMessage>, message: &MyMessage) {
			println!("Message {}", message.data);
		}
	}

	#[test]
	fn test_service_register_handler () {
		let mut service = IoService::<MyMessage>::start().expect("Error creating network service");
		service.register_handler(Arc::new(MyHandler)).unwrap();
	}

}
