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

//! Creates and registers client and network services.

use util::*;
use util::panics::*;
use spec::Spec;
use error::*;
use client::{Client, ClientConfig};

/// Message type for external and internal events
#[derive(Clone)]
pub enum SyncMessage {
	/// New block has been imported into the blockchain
	NewChainBlocks {
		/// Hashes of blocks imported to blockchain
		imported: Vec<H256>,
		/// Hashes of blocks not imported to blockchain (because were invalid)
		invalid: Vec<H256>,
		/// Hashes of blocks that were removed from canonical chain
		retracted: Vec<H256>,
		/// Hashes of blocks that are now included in cannonical chain
		enacted: Vec<H256>,
	},
	/// Best Block Hash in chain has been changed
	NewChainHead,
	/// A block is ready
	BlockVerified,
}

/// IO Message type used for Network service
pub type NetSyncMessage = NetworkIoMessage<SyncMessage>;

/// Client service setup. Creates and registers client and network services with the IO subsystem.
pub struct ClientService {
	net_service: NetworkService<SyncMessage>,
	client: Arc<Client>,
	panic_handler: Arc<PanicHandler>
}

impl ClientService {
	/// Start the service in a separate thread.
	pub fn start(config: ClientConfig, spec: Spec, net_config: NetworkConfiguration, db_path: &Path) -> Result<ClientService, Error> {
		let panic_handler = PanicHandler::new_in_arc();
		let mut net_service = try!(NetworkService::start(net_config));
		panic_handler.forward_from(&net_service);

		info!("Starting {}", net_service.host_info());
		info!("Configured for {} using {:?} engine", spec.name, spec.engine.name());
		let client = Client::new(config, spec, db_path, net_service.io().channel());
		panic_handler.forward_from(client.deref());
		let client_io = Arc::new(ClientIoHandler {
			client: client.clone()
		});
		try!(net_service.io().register_handler(client_io));

		Ok(ClientService {
			net_service: net_service,
			client: client,
			panic_handler: panic_handler,
		})
	}

	/// Add a node to network
	pub fn add_node(&mut self, _enode: &str) {
		unimplemented!();
	}

	/// Get general IO interface
	pub fn io(&mut self) -> &mut IoService<NetSyncMessage> {
		self.net_service.io()
	}

	/// Get client interface
	pub fn client(&self) -> Arc<Client> {
		self.client.clone()
	}

	/// Get network service component
	pub fn network(&mut self) -> &mut NetworkService<SyncMessage> {
		&mut self.net_service
	}
}

impl MayPanic for ClientService {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}

/// IO interface for the Client handler
struct ClientIoHandler {
	client: Arc<Client>
}

const CLIENT_TICK_TIMER: TimerToken = 0;
const CLIENT_TICK_MS: u64 = 5000;

impl IoHandler<NetSyncMessage> for ClientIoHandler {
	fn initialize(&self, io: &IoContext<NetSyncMessage>) {
		io.register_timer(CLIENT_TICK_TIMER, CLIENT_TICK_MS).expect("Error registering client timer");
	}

	fn timeout(&self, _io: &IoContext<NetSyncMessage>, timer: TimerToken) {
		if timer == CLIENT_TICK_TIMER {
			self.client.tick();
		}
	}

	#[cfg_attr(feature="dev", allow(single_match))]
	fn message(&self, io: &IoContext<NetSyncMessage>, net_message: &NetSyncMessage) {
		if let UserMessage(ref message) = *net_message {
			match *message {
				SyncMessage::BlockVerified => {
					self.client.import_verified_blocks(&io.channel());
				},
				_ => {}, // ignore other messages
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tests::helpers::*;
	use util::network::*;
	use devtools::*;
	use client::ClientConfig;

	#[test]
	fn it_can_be_started() {
		let spec = get_test_spec();
		let temp_path = RandomTempPath::new();
		let service = ClientService::start(ClientConfig::default(), spec, NetworkConfiguration::new_local(), &temp_path.as_path());
		assert!(service.is_ok());
	}
}
