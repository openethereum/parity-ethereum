/// General IO module.
///
/// Example usage for craeting a network service and adding an IO handler:
///
/// ```rust
/// extern crate ethcore_util;
/// use ethcore_util::*;
///
/// struct MyHandler;
///
/// struct MyMessage {
/// 	data: u32
/// }
///
///	impl IoHandler<MyMessage> for MyHandler {
///		fn initialize(&mut self, io: &mut IoContext<MyMessage>) {
///			io.register_timer(1000).unwrap();
///		}
///
///		fn timeout(&mut self, _io: &mut IoContext<MyMessage>, timer: TimerToken) {
///			println!("Timeout {}", timer);
///		}
///
///		fn message(&mut self, _io: &mut IoContext<MyMessage>, message: &mut MyMessage) {
///			println!("Message {}", message.data);
///		}
///	}
///
/// fn main () {
/// 	let mut service = IoService::<MyMessage>::start().expect("Error creating network service");
/// 	service.register_handler(Box::new(MyHandler)).unwrap();
///
/// 	// Wait for quit condition
/// 	// ...
/// 	// Drop the service
/// }
/// ```
mod service;
mod worker;

use mio::{EventLoop, Token};

#[derive(Debug)]
/// TODO [arkpar] Please document me
pub enum IoError {
	/// TODO [arkpar] Please document me
	Mio(::std::io::Error),
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
}

/// TODO [arkpar] Please document me
pub use io::service::TimerToken;
/// TODO [arkpar] Please document me
pub use io::service::StreamToken;
/// TODO [arkpar] Please document me
pub use io::service::IoContext;
/// TODO [arkpar] Please document me
pub use io::service::IoService;
/// TODO [arkpar] Please document me
pub use io::service::IoChannel;
/// TODO [arkpar] Please document me
pub use io::service::IoManager;
/// TODO [arkpar] Please document me
pub use io::service::TOKENS_PER_HANDLER;

#[cfg(test)]
mod tests {

	use io::*;

	struct MyHandler;

	struct MyMessage {
		data: u32
	}

	impl IoHandler<MyMessage> for MyHandler {
		fn initialize(&mut self, io: &mut IoContext<MyMessage>) {
			io.register_timer(1000).unwrap();
		}

		fn timeout(&mut self, _io: &mut IoContext<MyMessage>, timer: TimerToken) {
			println!("Timeout {}", timer);
		}

		fn message(&mut self, _io: &mut IoContext<MyMessage>, message: &mut MyMessage) {
			println!("Message {}", message.data);
		}
	}

	#[test]
	fn test_service_register_handler () {
		let mut service = IoService::<MyMessage>::start().expect("Error creating network service");
		service.register_handler(Box::new(MyHandler)).unwrap();
	}

}
