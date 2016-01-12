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

impl<M> From<::mio::NotifyError<service::IoMessage<M>>> for IoError {
	fn from(_err: ::mio::NotifyError<service::IoMessage<M>>) -> IoError {
		IoError::Mio(::std::io::Error::new(::std::io::ErrorKind::ConnectionAborted, "Network IO notification error"))
	}
}

pub type TimerToken = service::TimerToken;
pub type StreamToken = service::StreamToken;
pub type IoContext<'s, M> = service::IoContext<'s, M>;
pub type Message<M> = service::UserMessage<M>;
pub type IoService<M> = service::IoService<M>;
pub type IoHandler<M> = service::IoHandler<M>;



