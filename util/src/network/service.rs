use error::*;
use network::{NetworkProtocolHandler};
use network::error::{NetworkError};
use network::host::{Host, NetworkIoMessage, PeerId, PacketId, ProtocolId};
use io::*;

/// IO Service with networking
/// `Message` defines a notification data type.
pub struct NetworkService<Message> where Message: Send + 'static {
	io_service: IoService<NetworkIoMessage<Message>>,
	host_info: String,
}

impl<Message> NetworkService<Message> where Message: Send + 'static {
	/// Starts IO event loop
	pub fn start(init_nodes: &Vec<String>) -> Result<NetworkService<Message>, UtilError> {
		let mut io_service = try!(IoService::<NetworkIoMessage<Message>>::start());
		let mut host = Box::new(Host::new());
		for n in init_nodes { host.add_node(&n); }
		let host_info = host.info.client_version.clone();
		info!("NetworkService::start(): id={:?}", host.info.id());
		try!(io_service.register_handler(host));
		Ok(NetworkService {
			io_service: io_service,
			host_info: host_info,
		})
	}

	/// Send a message over the network. Normaly `HostIo::send` should be used. This can be used from non-io threads.
	pub fn send(&mut self, peer: &PeerId, packet_id: PacketId, protocol: ProtocolId, data: &[u8]) -> Result<(), NetworkError> {
		try!(self.io_service.send_message(NetworkIoMessage::Send {
			peer: *peer,
			packet_id: packet_id,
			protocol: protocol,
			data: data.to_vec()
		}));
		Ok(())
	}

	/// Regiter a new protocol handler with the event loop.
	pub fn register_protocol(&mut self, handler: Box<NetworkProtocolHandler<Message>+Send>, protocol: ProtocolId, versions: &[u8]) -> Result<(), NetworkError> {
		try!(self.io_service.send_message(NetworkIoMessage::AddHandler {
			handler: Some(handler),
			protocol: protocol,
			versions: versions.to_vec(),
		}));
		Ok(())
	}

	/// Returns host identifier string as advertised to other peers
	pub fn host_info(&self) -> String {
		self.host_info.clone()
	}

	/// Returns underlying io service.
	pub fn io(&mut self) -> &mut IoService<NetworkIoMessage<Message>> {
		&mut self.io_service
	}
}

