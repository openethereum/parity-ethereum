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
use network::error::{NetworkError};
use network::host::{Host, NetworkIoMessage, ProtocolId};
use network::stats::{NetworkStats};
use io::*;

/// IO Service with networking
/// `Message` defines a notification data type.
pub struct NetworkService<Message> where Message: Send + Sync + Clone + 'static {
	io_service: IoService<NetworkIoMessage<Message>>,
	host_info: String,
	host: Arc<Host<Message>>,
	stats: Arc<NetworkStats>,
	panic_handler: Arc<PanicHandler>
}

impl<Message> NetworkService<Message> where Message: Send + Sync + Clone + 'static {
	/// Starts IO event loop
	pub fn start(config: NetworkConfiguration) -> Result<NetworkService<Message>, UtilError> {
		let panic_handler = PanicHandler::new_in_arc();
		let mut io_service = try!(IoService::<NetworkIoMessage<Message>>::start());
		panic_handler.forward_from(&io_service);

		let host = Arc::new(try!(Host::new(config)));
		let stats = host.stats().clone();
		let host_info = host.client_version();
		try!(io_service.register_handler(host.clone()));
		Ok(NetworkService {
			io_service: io_service,
			host_info: host_info,
			stats: stats,
			panic_handler: panic_handler,
			host: host,
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

	/// Returns network statistics.
	pub fn stats(&self) -> &NetworkStats {
		&self.stats
	}

	/// Returns external url if available.
	pub fn external_url(&self) -> Option<String> {
		self.host.external_url()
	}

	/// Returns external url if available.
	pub fn local_url(&self) -> String {
		self.host.local_url()
	}
}

impl<Message> MayPanic for NetworkService<Message> where Message: Send + Sync + Clone + 'static {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}
