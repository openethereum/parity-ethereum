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

use std::sync::*;
use error::*;
use panics::*;
use network::{NetworkProtocolHandler, NetworkConfiguration};
use network::error::NetworkError;
use network::host::{Host, NetworkIoMessage, ProtocolId};
use network::stats::NetworkStats;
use io::*;

/// IO Service with networking
/// `Message` defines a notification data type.
pub struct NetworkService<Message> where Message: Send + Sync + Clone + 'static {
	io_service: IoService<NetworkIoMessage<Message>>,
	host_info: String,
	host: RwLock<Option<Arc<Host<Message>>>>,
	stats: Arc<NetworkStats>,
	panic_handler: Arc<PanicHandler>,
	config: NetworkConfiguration,
}

impl<Message> NetworkService<Message> where Message: Send + Sync + Clone + 'static {
	/// Starts IO event loop
	pub fn new(config: NetworkConfiguration) -> Result<NetworkService<Message>, UtilError> {
		let panic_handler = PanicHandler::new_in_arc();
		let io_service = try!(IoService::<NetworkIoMessage<Message>>::start());
		panic_handler.forward_from(&io_service);

		let stats = Arc::new(NetworkStats::new());
		let host_info = Host::<Message>::client_version();
		Ok(NetworkService {
			io_service: io_service,
			host_info: host_info,
			stats: stats,
			panic_handler: panic_handler,
			host: RwLock::new(None),
			config: config,
		})
	}

	/// Regiter a new protocol handler with the event loop.
	pub fn register_protocol(&self, handler: Arc<NetworkProtocolHandler<Message>+Send + Sync>, protocol: ProtocolId, versions: &[u8]) -> Result<(), NetworkError> {
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
	pub fn io(&self) -> &IoService<NetworkIoMessage<Message>> {
		&self.io_service
	}

	/// Returns network statistics.
	pub fn stats(&self) -> &NetworkStats {
		&self.stats
	}

	/// Returns external url if available.
	pub fn external_url(&self) -> Option<String> {
		let host = self.host.read().unwrap();
		host.as_ref().and_then(|h| h.external_url())
	}

	/// Returns external url if available.
	pub fn local_url(&self) -> Option<String> {
		let host = self.host.read().unwrap();
		host.as_ref().map(|h| h.local_url())
	}

	/// Start network IO
	pub fn start(&self) -> Result<(), UtilError> {
		let mut host = self.host.write().unwrap();
		if host.is_none() {
			let h = Arc::new(try!(Host::new(self.config.clone(), self.stats.clone())));
			try!(self.io_service.register_handler(h.clone()));
			*host = Some(h);
		}
		Ok(())
	}

	/// Stop network IO
	pub fn stop(&self) -> Result<(), UtilError> {
		let mut host = self.host.write().unwrap();
		if let Some(ref host) = *host {
			let io = IoContext::new(self.io_service.channel(), 0); //TODO: take token id from host
			try!(host.stop(&io));
		}
		*host = None;
		Ok(())
	}

	/// Try to add a reserved peer.
	pub fn add_reserved_peer(&self, peer: &str) -> Result<(), UtilError> {
		let host = self.host.read().unwrap();
		if let Some(ref host) = *host {
			host.add_reserved_node(peer)
		} else {
			Ok(())
		}
	}

	/// Try to remove a reserved peer.
	pub fn remove_reserved_peer(&self, peer: &str) -> Result<(), UtilError> {
		let host = self.host.read().unwrap();
		if let Some(ref host) = *host {
			host.remove_reserved_node(peer)
		} else {
			Ok(())
		}
	}

	/// Set the non-reserved peer mode.
	pub fn set_non_reserved_mode(&self, mode: ::network::NonReservedPeerMode) {
		let host = self.host.read().unwrap();
		if let Some(ref host) = *host {
			let io_ctxt = IoContext::new(self.io_service.channel(), 0);
			host.set_non_reserved_mode(mode, &io_ctxt);
		}
	}
}

impl<Message> MayPanic for NetworkService<Message> where Message: Send + Sync + Clone + 'static {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}
