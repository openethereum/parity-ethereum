// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use {NetworkProtocolHandler, NetworkConfiguration, NonReservedPeerMode};
use error::NetworkError;
use host::{Host, NetworkContext, NetworkIoMessage, PeerId, ProtocolId};
use stats::NetworkStats;
use io::*;
use parking_lot::RwLock;
use std::sync::Arc;
use ansi_term::Colour;
use connection_filter::ConnectionFilter;

struct HostHandler {
	public_url: RwLock<Option<String>>
}

impl IoHandler<NetworkIoMessage> for HostHandler {
	fn message(&self, _io: &IoContext<NetworkIoMessage>, message: &NetworkIoMessage) {
		if let NetworkIoMessage::NetworkStarted(ref public_url) = *message {
			let mut url = self.public_url.write();
			if url.as_ref().map_or(true, |uref| uref != public_url) {
				info!(target: "network", "Public node URL: {}", Colour::White.bold().paint(public_url.as_ref()));
			}
			*url = Some(public_url.to_owned());
		}
	}
}

/// IO Service with networking
/// `Message` defines a notification data type.
pub struct NetworkService {
	io_service: IoService<NetworkIoMessage>,
	host_info: String,
	host: RwLock<Option<Arc<Host>>>,
	stats: Arc<NetworkStats>,
	host_handler: Arc<HostHandler>,
	config: NetworkConfiguration,
	filter: Option<Arc<ConnectionFilter>>,
}

impl NetworkService {
	/// Starts IO event loop
	pub fn new(config: NetworkConfiguration, filter: Option<Arc<ConnectionFilter>>) -> Result<NetworkService, NetworkError> {
		let host_handler = Arc::new(HostHandler { public_url: RwLock::new(None) });
		let io_service = IoService::<NetworkIoMessage>::start()?;

		let stats = Arc::new(NetworkStats::new());
		let host_info = Host::client_version();
		Ok(NetworkService {
			io_service: io_service,
			host_info: host_info,
			stats: stats,
			host: RwLock::new(None),
			config: config,
			host_handler: host_handler,
			filter: filter,
		})
	}

	/// Regiter a new protocol handler with the event loop.
	pub fn register_protocol(&self, handler: Arc<NetworkProtocolHandler + Send + Sync>, protocol: ProtocolId, packet_count: u8, versions: &[u8]) -> Result<(), NetworkError> {
		self.io_service.send_message(NetworkIoMessage::AddHandler {
			handler: handler,
			protocol: protocol,
			versions: versions.to_vec(),
			packet_count: packet_count,
		})?;
		Ok(())
	}

	/// Returns host identifier string as advertised to other peers
	pub fn host_info(&self) -> String {
		self.host_info.clone()
	}

	/// Returns underlying io service.
	pub fn io(&self) -> &IoService<NetworkIoMessage> {
		&self.io_service
	}

	/// Returns network statistics.
	pub fn stats(&self) -> &NetworkStats {
		&self.stats
	}

	/// Returns network configuration.
	pub fn config(&self) -> &NetworkConfiguration {
		&self.config
	}

	/// Returns external url if available.
	pub fn external_url(&self) -> Option<String> {
		let host = self.host.read();
		host.as_ref().and_then(|h| h.external_url())
	}

	/// Returns external url if available.
	pub fn local_url(&self) -> Option<String> {
		let host = self.host.read();
		host.as_ref().map(|h| h.local_url())
	}

	/// Start network IO
	pub fn start(&self) -> Result<(), NetworkError> {
		let mut host = self.host.write();
		if host.is_none() {
			let h = Arc::new(Host::new(self.config.clone(), self.stats.clone(), self.filter.clone())?);
			self.io_service.register_handler(h.clone())?;
			*host = Some(h);
		}

		if self.host_handler.public_url.read().is_none() {
			self.io_service.register_handler(self.host_handler.clone())?;
		}

		Ok(())
	}

	/// Stop network IO
	pub fn stop(&self) -> Result<(), NetworkError> {
		let mut host = self.host.write();
		if let Some(ref host) = *host {
			let io = IoContext::new(self.io_service.channel(), 0); //TODO: take token id from host
			host.stop(&io)?;
		}
		*host = None;
		Ok(())
	}

	/// Get a list of all connected peers by id.
	pub fn connected_peers(&self) -> Vec<PeerId> {
		self.host.read().as_ref().map(|h| h.connected_peers()).unwrap_or_else(Vec::new)
	}

	/// Try to add a reserved peer.
	pub fn add_reserved_peer(&self, peer: &str) -> Result<(), NetworkError> {
		let host = self.host.read();
		if let Some(ref host) = *host {
			host.add_reserved_node(peer)
		} else {
			Ok(())
		}
	}

	/// Try to remove a reserved peer.
	pub fn remove_reserved_peer(&self, peer: &str) -> Result<(), NetworkError> {
		let host = self.host.read();
		if let Some(ref host) = *host {
			host.remove_reserved_node(peer)
		} else {
			Ok(())
		}
	}

	/// Set the non-reserved peer mode.
	pub fn set_non_reserved_mode(&self, mode: NonReservedPeerMode) {
		let host = self.host.read();
		if let Some(ref host) = *host {
			let io_ctxt = IoContext::new(self.io_service.channel(), 0);
			host.set_non_reserved_mode(mode, &io_ctxt);
		}
	}

	/// Executes action in the network context
	pub fn with_context<F>(&self, protocol: ProtocolId, action: F) where F: FnOnce(&NetworkContext) {
		let io = IoContext::new(self.io_service.channel(), 0);
		let host = self.host.read();
		if let Some(ref host) = host.as_ref() {
			host.with_context(protocol, &io, action);
		};
	}

	/// Evaluates function in the network context
	pub fn with_context_eval<F, T>(&self, protocol: ProtocolId, action: F) -> Option<T> where F: FnOnce(&NetworkContext) -> T {
		let io = IoContext::new(self.io_service.channel(), 0);
		let host = self.host.read();
		host.as_ref().map(|ref host| host.with_context_eval(protocol, &io, action))
	}
}
