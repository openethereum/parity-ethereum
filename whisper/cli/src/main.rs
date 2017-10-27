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

extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate parity_whisper;
extern crate panic_hook;
extern crate ethcore_network as net;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server as http;
extern crate jsonrpc_minihttp_server as minihttp;
extern crate jsonrpc_pubsub;

use std::{env, fmt, process};
use docopt::Docopt;
use std::io;
use net::*;
use parity_whisper::rpc::Whisper;

// const DAPPS_DOMAIN: &'static str = "web3.site";

use std::sync::Arc;
use parity_whisper::net::{self as whisper_net, Network as WhisperNetwork};
use parity_whisper::rpc::{WhisperClient, FilterManager, PoolHandle, Meta as WhisperMetadata};
use parity_whisper::message::Message;

pub struct WhisperPoolHandle {
	/// Pool handle.
	handle: Arc<WhisperNetwork<Arc<FilterManager>>>,
	/// Network manager.
	net: Arc<NetworkService>,
}

impl PoolHandle for WhisperPoolHandle {
	fn relay(&self, message: Message) -> bool {
		let mut res = false;
		let mut message = Some(message);
		self.with_proto_context(whisper_net::PROTOCOL_ID, &mut move |ctx| {
			if let Some(message) = message.take() {
				res = self.handle.post_message(message, ctx);
			}
		});
		res
	}

	fn pool_status(&self) -> whisper_net::PoolStatus {
		self.handle.pool_status()
	}
}

impl WhisperPoolHandle {
	fn with_proto_context(&self, proto: ProtocolId, f: &mut FnMut(&NetworkContext)) {
		self.net.with_context_eval(proto, f);
	}
}

pub struct RpcFactory {
	handle: Arc<WhisperNetwork<Arc<FilterManager>>>,
	manager: Arc<FilterManager>,
}

impl RpcFactory {
	pub fn make_handler(&self, net: Arc<NetworkService>) -> WhisperClient<NetPoolHandle, Metadata> {
		let whisper_pool_handle = WhisperPoolHandle { handle: self.handle.clone(), net: net.clone() };
		let whisper_rpc_handler : WhisperClient<WhisperPoolHandle, WhisperMetadata> = WhisperClient::new(whisper_pool_handle, self.manager.clone());
		whisper_rpc_handler
	}
}

// use ethsync::api::{SyncClient, NetworkManagerClient};

// use parity_rpc::*;

#[derive(Debug, Clone, PartialEq)]
pub struct HttpConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	// pub apis: ApiSet,
	pub cors: Option<Vec<String>>,
	pub hosts: Option<Vec<String>>,
	pub server_threads: Option<usize>,
	pub processing_threads: usize,
}

impl HttpConfiguration {
	pub fn address(&self) -> Option<(String, u16)> {
		match self.enabled {
			true => Some((self.interface.clone(), self.port)),
			false => None,
		}
	}
}

impl Default for HttpConfiguration {
	fn default() -> Self {
		HttpConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8545,
			// apis: ApiSet::UnsafeContext,
			cors: None,
			hosts: Some(Vec::new()),
			server_threads: None,
			processing_threads: 0,
		}
	}
}

pub const USAGE: &'static str = r#"
Whisper.
  Copyright 2017 Parity Technologies (UK) Ltd

Usage:
	whisper [options]
    whisper [-h | --help]

Options:
    --whisper-pool-size SIZE       Specify Whisper pool size [default: 10].
    -h, --help                     Display this message and exit.
"#;

// TODO: move to clap?

#[derive(Debug, Deserialize)]
struct Args {
	flag_whisper_pool_size: usize,
}

// TODO error-chain
#[derive(Debug)]
enum Error {
	// Whisper(WhisperError),
	Docopt(docopt::Error),
	Io(io::Error),
	MiniHttp(minihttp::Error)
}

// impl From<WhisperError> for Error {
// 	fn from(err: WhisperError) -> Self {
// 		Error::Whisper(err)
// 	}
// }

impl From<docopt::Error> for Error {
	fn from(err: docopt::Error) -> Self {
		Error::Docopt(err)
	}
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		Error::Io(err)
	}
}

impl From<minihttp::Error> for Error {
	fn from(err: minihttp::Error) -> Self {
		Error::MiniHttp(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			// Error::Whisper(ref e) => write!(f, "{}", e),
			Error::Docopt(ref e) => write!(f, "{}", e),
			Error::Io(ref e) => write!(f, "{}", e),
			Error::MiniHttp(ref e) => write!(f, "{:?}", e),
		}
	}
}

fn main() {
	panic_hook::set();

	match execute(env::args()) {
		Ok(ok) => println!("{}", ok),
		Err(err) => {
			println!("{}", err);
			process::exit(1);
		},
	}
}

fn execute<S, I>(command: I) -> Result<String, Error> where I: IntoIterator<Item=S>, S: AsRef<str> {

	// -- CLI Parsing
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.argv(command).deserialize())?;
	let target_message_pool_size = args.flag_whisper_pool_size * 1024 * 1024;

	// -- 1) Instantiate Whisper network handler
	let whisper_filter_manager = Arc::new(FilterManager::new()?);
	let whisper_network_handler = Arc::new(WhisperNetwork::new(target_message_pool_size, whisper_filter_manager.clone()));

	// -- 2) Instantiate Whisper network and attach to it the network handler
	let mut network = NetworkService::new(NetworkConfiguration::new_local(), None).expect("Error creating network service");
	network.start().expect("Error starting service");
	network.register_protocol(whisper_network_handler.clone(), whisper_net::PROTOCOL_ID, whisper_net::PACKET_COUNT, whisper_net::SUPPORTED_VERSIONS);
	network.register_protocol(Arc::new(whisper_net::ParityExtensions), whisper_net::PARITY_PROTOCOL_ID, whisper_net::PACKET_COUNT, whisper_net::SUPPORTED_VERSIONS);

	// -- 3) Instantiate RPC Handler
	// let whisper_pool_handle = WhisperPoolHandle { handle: whisper_network_handler.clone(), net: Arc::new(network) };
	// let whisper_rpc_handler : WhisperClient<WhisperPoolHandle, WhisperMetadata> = WhisperClient::new(whisper_pool_handle, whisper_filter_manager.clone());

	let whisper_factory = RpcFactory { handle: whisper_network_handler, manager: whisper_filter_manager };
	let mut rpc_handler : jsonrpc_core::MetaIoHandler<WhisperMetadata, _> = jsonrpc_core::MetaIoHandler::default();

	rpc_handler.extend_with(::parity_whisper::rpc::Whisper::to_delegate(whisper_factory.make_handler(Arc::new(network))));
	rpc_handler.extend_with(::parity_whisper::rpc::WhisperPubSub::to_delegate(whisper_factory.make_handler(Arc::new(network))));
	/*

					if let Some(ref whisper_rpc) = self.whisper_rpc {
						let whisper = whisper_rpc.make_handler(self.net.clone());
						handler.extend_with(::parity_whisper::rpc::WhisperPubSub::to_delegate(whisper));
					}

					donc utiliser factory
	*/


	// TODO WhisperPubSub?

	// -- 4) Launch RPC with handler

	// Api::WhisperPubSub
	// if !for_generic_pubsub {
		// needs arc managenetwork
	// handler.extend_with(
	// 	::parity_whisper::rpc::WhisperPubSub::to_delegate(whisper_pubsub_handler) // not sure why
	// );
	// }

	let mut http_configuration = HttpConfiguration::default();
	let http_address = http_configuration.address().unwrap();
	let url = format!("{}:{}", http_address.0, http_address.1);
	let addr = url.parse().map_err(|_| format!("Invalid listen host/port given: {}", url)); // "?"

	// let mut allowed_hosts: Option<Vec<Host>> = http_configuration.hosts.into();
	// allowed_hosts.as_mut().map(|mut hosts| {


	// 	hosts.push(format!("http://*.{}:*", DAPPS_DOMAIN).into());
	// 	hosts.push(format!("http://*.{}", DAPPS_DOMAIN).into());
	// });

	let threads = 1;
	let server = minihttp::ServerBuilder::new(rpc_handler) // yay handler => rpc
				.threads(threads) // config param I guess // todo httpconfiguration
				// .meta_extractor(http_common::MiniMetaExtractor::new(extractor))
				// .cors(http_configuration.cors.into()) // cli
				//.allowed_hosts(allowed_hosts.into()) // cli
				.start_http(&addr.unwrap())
				.expect("Unable to start RPC server");
				// .map(HttpServer::Mini)?;

	println!("Rpc about to listen.");
	server.wait().unwrap();
	println!("Rpc listening.");
	// Arc::new(whisper_net::ParityExtensions),
	// ou bien via factory

// let deps_for_rpc_apis = Arc::new(rpc_apis::FullDependencies {
// 		signer_service: signer_service,
// 		snapshot: snapshot_service.clone(),
// 		client: client.clone(),
// 		sync: sync_provider.clone(),
// 		health: node_health,
// 		net: manage_network.clone(),
// 		secret_store: secret_store,
// 		miner: miner.clone(),
// 		external_miner: external_miner.clone(),
// 		logger: logger.clone(),
// 		settings: Arc::new(cmd.net_settings.clone()),
// 		net_service: manage_network.clone(),
// 		updater: updater.clone(),
// 		geth_compatibility: cmd.geth_compatibility,
// 		dapps_service: dapps_service,
// 		dapps_address: cmd.dapps_conf.address(cmd.http_conf.address()),
// 		ws_address: cmd.ws_conf.address(),
// 		fetch: fetch.clone(),
// 		remote: event_loop.remote(),
// 		whisper_rpc: whisper_factory,
// 	}); //<-- I don't need this










	// FROM HERE ON, clean / ok / goody


// let mut builder = http::ServerBuilder::new(handler)
// 				.event_loop_remote(remote)
// 				.meta_extractor(http_common::HyperMetaExtractor::new(extractor))
// 				.cors(cors_domains.into())
// 				.allowed_hosts(allowed_hosts.into());

// 			if let Some(dapps) = middleware {
// 				builder = builder.request_middleware(dapps)
// 			}
// 			builder.start_http(addr)
// 				.map(HttpServer::Hyper)?












	/*
		util/network/src/networkconfigration
		create netwokconfiguration, crete network ervice (lib.rs)
	*/

	// let (whisper_net, whisper_factory) = ::whisper::setup(target_message_pool_size)
	// 	.map_err(|e| format!("Failed to initialize whisper: {}", e))?;

	// attached_protos.push(whisper_net);

	Ok("OK".to_owned())
}















// #[cfg(not(feature = "ipc"))]
// pub fn whisper_setup(target_pool_size: usize, protos: &mut Vec<AttachedProtocol>)
// 	-> io::Result<Option<RpcFactory>> // whisper::RpcFactory
// {
// 	let manager = Arc::new(FilterManager::new()?);
// 	let net = Arc::new(WhisperNetwork::new(target_pool_size, manager.clone()));

// 	protos.push(AttachedProtocol {
// 		handler: net.clone() as Arc<_>,
// 		packet_count: whisper_net::PACKET_COUNT,
// 		versions: whisper_net::SUPPORTED_VERSIONS,
// 		protocol_id: whisper_net::PROTOCOL_ID,
// 	});

// 	// parity-only extensions to whisper.
// 	protos.push(AttachedProtocol {
// 		handler: Arc::new(whisper_net::ParityExtensions),
// 		packet_count: whisper_net::PACKET_COUNT,
// 		versions: whisper_net::SUPPORTED_VERSIONS,
// 		protocol_id: whisper_net::PARITY_PROTOCOL_ID,
// 	});

// 	let factory = RpcFactory { net: net, manager: manager };

// 	Ok(Some(factory))
// }

// // TODO: make it possible to attach generic protocols in IPC.
// #[cfg(feature = "ipc")]
// pub fn whisper_setup(_target_pool_size: usize, _protos: &mut Vec<AttachedProtocol>)
// 	-> io::Result<Option<RpcFactory>>
// {
// 	Ok(None)
// }
