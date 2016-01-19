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

#[derive(Debug)]
/// TODO [arkpar] Please document me
pub enum IoError {
	/// TODO [arkpar] Please document me
	Mio(::std::io::Error),
}

impl<Message> From<::mio::NotifyError<service::IoMessage<Message>>> for IoError where Message: Send {
	fn from(_err: ::mio::NotifyError<service::IoMessage<Message>>) -> IoError {
		IoError::Mio(::std::io::Error::new(::std::io::ErrorKind::ConnectionAborted, "Network IO notification error"))
	}
}

/// Generic IO handler. 
/// All the handler function are called from within IO event loop.
/// `Message` type is used as notification data
pub trait IoHandler<Message>: Send where Message: Send + 'static {
	/// Initialize the handler
	fn initialize<'s>(&'s mut self, _io: &mut IoContext<'s, Message>) {}
	/// Timer function called after a timeout created with `HandlerIo::timeout`.
	fn timeout<'s>(&'s mut self, _io: &mut IoContext<'s, Message>, _timer: TimerToken) {}
	/// Called when a broadcasted message is received. The message can only be sent from a different IO handler.
	fn message<'s>(&'s mut self, _io: &mut IoContext<'s, Message>, _message: &'s mut Message) {} // TODO: make message immutable and provide internal channel for adding network handler
	/// Called when an IO stream gets closed
	fn stream_hup<'s>(&'s mut self, _io: &mut IoContext<'s, Message>, _stream: StreamToken) {}
	/// Called when an IO stream can be read from 
	fn stream_readable<'s>(&'s mut self, _io: &mut IoContext<'s, Message>, _stream: StreamToken) {}
	/// Called when an IO stream can be written to
	fn stream_writable<'s>(&'s mut self, _io: &mut IoContext<'s, Message>, _stream: StreamToken) {}
}

/// TODO [arkpar] Please document me
pub type TimerToken = service::TimerToken;
/// TODO [arkpar] Please document me
pub type StreamToken = service::StreamToken;
/// TODO [arkpar] Please document me
pub type IoContext<'s, M> = service::IoContext<'s, M>;
/// TODO [arkpar] Please document me
pub type IoService<M> = service::IoService<M>;
/// TODO [arkpar] Please document me
pub type IoChannel<M> = service::IoChannel<M>;
//pub const USER_TOKEN_START: usize = service::USER_TOKEN; // TODO: ICE in rustc 1.7.0-nightly (49c382779 2016-01-12)

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
