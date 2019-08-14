// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::sync::Arc;

use ansi_term::Colour;
use log::info;
use parking_lot::RwLock;

use ethcore_io::{IoContext, IoHandler, IoService};
use network::{
	ConnectionFilter, Error, NetworkConfiguration, NetworkContext,
	NetworkIoMessage, NetworkProtocolHandler, NonReservedPeerMode, PeerId, ProtocolId,

};

use crate::host::Host;

struct HostHandler {
	public_url: RwLock<Option<String>>
}

impl IoHandler<NetworkIoMessage> for HostHandler {
	fn message(&self, _io: &IoContext<NetworkIoMessage>, message: &NetworkIoMessage) {
		if let NetworkIoMessage::NetworkStarted(ref public_url) = *message {
			let mut url = self.public_url.write();
			if url.as_ref().map_or(true, |uref| uref != public_url) {
				info!(target: "network", "Public node URL: {}", Colour::White.bold().paint(AsRef::<str>::as_ref(public_url)));
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
	host_handler: Arc<HostHandler>,
	config: NetworkConfiguration,
	filter: Option<Arc<dyn ConnectionFilter>>,
}

impl NetworkService {
	/// Starts IO event loop
	pub fn new(config: NetworkConfiguration, filter: Option<Arc<dyn ConnectionFilter>>) -> Result<NetworkService, Error> {
		let host_handler = Arc::new(HostHandler { public_url: RwLock::new(None) });
		let io_service = IoService::<NetworkIoMessage>::start()?;

		Ok(NetworkService {
			io_service,
			host_info: config.client_version.clone(),
			host: RwLock::new(None),
			config,
			host_handler,
			filter,
		})
	}

	/// Register a new protocol handler with the event loop.
	pub fn register_protocol(
		&self,
		handler: Arc<dyn NetworkProtocolHandler + Send + Sync>,
		protocol: ProtocolId,
		// version id + packet count
		versions: &[(u8, u8)]
	) -> Result<(), Error> {
		self.io_service.send_message(NetworkIoMessage::AddHandler {
			handler,
			protocol,
			versions: versions.to_vec(),
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

	/// Returns the number of peers allowed.
	pub fn num_peers_range(&self) -> RangeInclusive<u32> {
		self.config.min_peers..=self.config.max_peers
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

	/// Start network IO.
	///
	/// In case of error, also returns the listening address for better error reporting.
	pub fn start(&self) -> Result<(), (Error, Option<SocketAddr>)> {
		let mut host = self.host.write();
		let listen_addr = self.config.listen_address;
		if host.is_none() {
			let h = Arc::new(Host::new(self.config.clone(), self.filter.clone())
				.map_err(|err| (err, listen_addr))?);
			self.io_service.register_handler(h.clone())
				.map_err(|err| (err.into(), listen_addr))?;
			*host = Some(h);
		}

		if self.host_handler.public_url.read().is_none() {
			self.io_service.register_handler(self.host_handler.clone())
				.map_err(|err| (err.into(), listen_addr))?;
		}

		Ok(())
	}

	/// Stop network IO.
	pub fn stop(&self) {
		let mut host = self.host.write();
		if let Some(ref host) = *host {
			let io = IoContext::new(self.io_service.channel(), 0); //TODO: take token id from host
			host.stop(&io);
		}
		*host = None;
	}

	/// Get a list of all connected peers by id.
	pub fn connected_peers(&self) -> Vec<PeerId> {
		self.host.read().as_ref().map(|h| h.connected_peers()).unwrap_or_else(Vec::new)
	}

	/// Try to add a reserved peer.
	pub fn add_reserved_peer(&self, peer: &str) -> Result<(), Error> {
		let host = self.host.read();
		if let Some(ref host) = *host {
			host.add_reserved_node(peer)
		} else {
			Ok(())
		}
	}

	/// Try to remove a reserved peer.
	pub fn remove_reserved_peer(&self, peer: &str) -> Result<(), Error> {
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
	pub fn with_context<F>(&self, protocol: ProtocolId, action: F) where F: FnOnce(&dyn NetworkContext) {
		let io = IoContext::new(self.io_service.channel(), 0);
		let host = self.host.read();
		if let Some(ref host) = host.as_ref() {
			host.with_context(protocol, &io, action);
		};
	}

	/// Evaluates function in the network context
	pub fn with_context_eval<F, T>(&self, protocol: ProtocolId, action: F) -> Option<T> where F: FnOnce(&dyn NetworkContext) -> T {
		let io = IoContext::new(self.io_service.channel(), 0);
		let host = self.host.read();
		host.as_ref().map(|ref host| host.with_context_eval(protocol, &io, action))
	}
}
