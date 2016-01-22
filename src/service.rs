use util::*;
use sync::*;
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
	sync: Arc<EthSync>,
}

impl ClientService {
	/// Start the service in a separate thread.
	pub fn start(spec: Spec) -> Result<ClientService, Error> {
		let mut net_service = try!(NetworkService::start());
		info!("Starting {}", net_service.host_info());
		info!("Configured for {} using {} engine", spec.name, spec.engine_name);
		let mut dir = env::home_dir().unwrap();
		dir.push(".parity");
		dir.push(H64::from(spec.genesis_header().hash()).hex());
		let client = try!(Client::new(spec, &dir, net_service.io().channel()));
		let sync = EthSync::register(&mut net_service, client.clone());
		let client_io = Arc::new(ClientIoHandler {
			client: client.clone()
		});
		try!(net_service.io().register_handler(client_io));

		Ok(ClientService {
			net_service: net_service,
			client: client,
			sync: sync,
		})
	}

	/// TODO [arkpar] Please document me
	pub fn io(&mut self) -> &mut IoService<NetSyncMessage> {
		self.net_service.io()
	}

	/// TODO [arkpar] Please document me
	pub fn client(&self) -> Arc<Client> {
		self.client.clone()
	
	}
	
	/// Get shared sync handler
	pub fn sync(&self) -> Arc<EthSync> {
		self.sync.clone()
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

	fn message(&self, io: &IoContext<NetSyncMessage>, net_message: &NetSyncMessage) {
		match net_message {
			&UserMessage(ref message) =>  {
				match message {
					&SyncMessage::BlockVerified => {
						self.client.import_verified_blocks(&io.channel());
					},
					_ => {}, // ignore other messages
				}

			}
			_ => {}, // ignore other messages
		}

	}
}

