//! Client service.
//!
//!
//!
//!
//!

use util::*;
use sync::*;
use spec::Spec;
use error::*;
use std::env;
use client::Client;

pub struct ClientService {
	_net_service: NetworkService<SyncMessage>,
}

impl ClientService {
	pub fn start(spec: Spec) -> Result<ClientService, Error> {
		let mut net_service = try!(NetworkService::start());
		info!("Starting {}", net_service.host_info());
		info!("Configured for {} using {} engine", spec.name, spec.engine_name);
		let mut dir = env::home_dir().unwrap();
		dir.push(".parity");
		dir.push(H64::from(spec.genesis_header().hash()).hex());
		let client = Arc::new(RwLock::new(try!(Client::new(spec, &dir, net_service.io().channel()))));
		EthSync::register(&mut net_service, client.clone());
		let client_io = Box::new(ClientIoHandler {
			client: client
		});
		try!(net_service.io().register_handler(client_io));

		Ok(ClientService {
			_net_service: net_service,
		})
	}
}

struct ClientIoHandler {
	client: Arc<RwLock<Client>>
}

impl IoHandler<NetSyncMessage> for ClientIoHandler {
	fn initialize<'s>(&'s mut self, _io: &mut IoContext<'s, NetSyncMessage>) {
	}

	fn message<'s>(&'s mut self, _io: &mut IoContext<'s, NetSyncMessage>, net_message: &'s mut NetSyncMessage) {
		match net_message {
			&mut UserMessage(ref mut message) =>  {
				match message {
					&mut SyncMessage::BlockVerified(ref mut bytes) => {
						self.client.write().unwrap().import_verified_block(mem::replace(bytes, Bytes::new()));
					},
					_ => {},
				}

			}
			_ => {},
		}

	}
}

