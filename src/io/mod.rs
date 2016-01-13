/// General IO module.
///
/// Example usage for craeting a network service and adding an IO handler:
///
/// ```rust
/// extern crate ethcore_util as util;
/// use util::network::*;
///
/// struct MyHandler;
///
/// impl ProtocolHandler for MyHandler {
///		fn initialize(&mut self, io: &mut HandlerIo) {
///			io.register_timer(1000);
///		}
///
///		fn read(&mut self, io: &mut HandlerIo, peer: &PeerId, packet_id: u8, data: &[u8]) {
///			println!("Received {} ({} bytes) from {}", packet_id, data.len(), peer);
///		}
///
///		fn connected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
///			println!("Connected {}", peer);
///		}
///
///		fn disconnected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
///			println!("Disconnected {}", peer);
///		}
///
///		fn timeout(&mut self, io: &mut HandlerIo, timer: TimerToken) {
///			println!("Timeout {}", timer);
///		}
///
///		fn message(&mut self, io: &mut HandlerIo, message: &Message) {
///			println!("Message {}:{}", message.protocol, message.id);
///		}
/// }
///
/// fn main () {
/// 	let mut service = NetworkService::start().expect("Error creating network service");
/// 	service.register_protocol(Box::new(MyHandler), "myproto", &[1u8]);
///
/// 	// Wait for quit condition
/// 	// ...
/// 	// Drop the service
/// }
/// ```
extern crate mio;
mod service;

#[derive(Debug)]
pub enum IoError {
	Mio(::std::io::Error),
}

impl<M> From<::mio::NotifyError<service::IoMessage<M>>> for IoError where M: Send {
	fn from(_err: ::mio::NotifyError<service::IoMessage<M>>) -> IoError {
		IoError::Mio(::std::io::Error::new(::std::io::ErrorKind::ConnectionAborted, "Network IO notification error"))
	}
}

/// Generic IO handler. 
/// All the handler function are called from within IO event loop.
/// `Message` type is used as notification data
pub trait IoHandler<Message>: Send where Message: Send + 'static {
	/// Initialize the handler
	fn initialize(&mut self, _io: IoContext<Message>) {}
	/// Timer function called after a timeout created with `HandlerIo::timeout`.
	fn timeout(&mut self, _io: IoContext<Message>, _timer: TimerToken) {}
	/// Called when a broadcasted message is received. The message can only be sent from a different IO handler.
	fn message(&mut self, _io: IoContext<Message>, _message: &mut Message) {} // TODO: make message immutable and provide internal channel for adding network handler
	/// Called when an IO stream gets closed
	fn stream_hup(&mut self, _io: IoContext<Message>, _stream: StreamToken) {}
	/// Called when an IO stream can be read from 
	fn stream_readable(&mut self, _io: IoContext<Message>, _stream: StreamToken) {}
	/// Called when an IO stream can be written to
	fn stream_writable(&mut self, _io: IoContext<Message>, _stream: StreamToken) {}
}

pub type TimerToken = service::TimerToken;
pub type StreamToken = service::StreamToken;
pub type IoContext<'s, M> = service::IoContext<'s, M>;
pub type IoService<M> = service::IoService<M>;
pub const USER_TOKEN_START: usize = service::USER_TOKEN;



