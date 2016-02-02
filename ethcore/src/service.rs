//! Creates and registers client and network services.

use util::*;
use spec::Spec;
use error::*;
use std::env;
use client::Client;

/// Message type for external and internal events
#[derive(Clone)]
pub enum SyncMessage {
	/// New block has been imported into the blockchain
	NewChainBlock(Bytes), //TODO: use Cow
	/// A block is ready 
	BlockVerified,
}

/// TODO [arkpar] Please document me
pub type NetSyncMessage = NetworkIoMessage<SyncMessage>;

/// Client service setup. Creates and registers client and network services with the IO subsystem.
pub struct ClientService {
	net_service: NetworkService<SyncMessage>,
	client: Arc<Client>,
}

impl ClientService {
	/// Start the service in a separate thread.
	pub fn start(spec: Spec, net_config: NetworkConfiguration) -> Result<ClientService, Error> {
		let mut net_service = try!(NetworkService::start(net_config));
		info!("Starting {}", net_service.host_info());
		info!("Configured for {} using {} engine", spec.name, spec.engine_name);
		let mut dir = env::home_dir().unwrap();
		dir.push(".parity");
		dir.push(H64::from(spec.genesis_header().hash()).hex());
		let client = try!(Client::new(spec, &dir, net_service.io().channel()));
		let client_io = Arc::new(ClientIoHandler {
			client: client.clone()
		});
		try!(net_service.io().register_handler(client_io));

		Ok(ClientService {
			net_service: net_service,
			client: client,
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

	#[allow(match_ref_pats)]
	#[allow(single_match)]
	fn message(&self, io: &IoContext<NetSyncMessage>, net_message: &NetSyncMessage) {
		if let &UserMessage(ref message) = net_message {
			match message {
				&SyncMessage::BlockVerified => {
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

	#[test]
	fn it_can_be_started() {
		let spec = get_test_spec();
		let service = ClientService::start(spec, NetworkConfiguration::new());
		assert!(service.is_ok());
	}
}
