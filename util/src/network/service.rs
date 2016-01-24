use std::sync::*;
use error::*;
use network::{NetworkProtocolHandler, NetworkConfiguration};
use network::error::{NetworkError};
use network::host::{Host, NetworkIoMessage, ProtocolId};
use network::stats::{NetworkStats};
use io::*;

/// IO Service with networking
/// `Message` defines a notification data type.
pub struct NetworkService<Message> where Message: Send + Sync + Clone + 'static {
	io_service: IoService<NetworkIoMessage<Message>>,
	host_info: String,
	stats: Arc<NetworkStats>
}

impl<Message> NetworkService<Message> where Message: Send + Sync + Clone + 'static {
	/// Starts IO event loop
	pub fn start(config: NetworkConfiguration) -> Result<NetworkService<Message>, UtilError> {
		let mut io_service = try!(IoService::<NetworkIoMessage<Message>>::start());
		let host = Arc::new(Host::new(config));
		let stats = host.stats().clone();
		let host_info = host.client_version();
		info!("NetworkService::start(): id={:?}", host.client_id());
		try!(io_service.register_handler(host));
		Ok(NetworkService {
			io_service: io_service,
			host_info: host_info,
			stats: stats,
		})
	}

	/// Regiter a new protocol handler with the event loop.
	pub fn register_protocol(&mut self, handler: Arc<NetworkProtocolHandler<Message>+Send + Sync>, protocol: ProtocolId, versions: &[u8]) -> Result<(), NetworkError> {
		try!(self.io_service.send_message(NetworkIoMessage::AddHandler {
			handler: handler,
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

	/// Returns underlying io service.
	pub fn stats(&self) -> &NetworkStats {
		&self.stats
	}
}

