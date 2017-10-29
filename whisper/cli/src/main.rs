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
	pub fn make_handler(&self, net: Arc<NetworkService>) -> WhisperClient<WhisperPoolHandle, WhisperMetadata> {
		let whisper_pool_handle = WhisperPoolHandle { handle: self.handle.clone(), net: net };
		let whisper_rpc_handler : WhisperClient<WhisperPoolHandle, WhisperMetadata> = WhisperClient::new(whisper_pool_handle, self.manager.clone());
		whisper_rpc_handler
	}
}

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

// TODO error-chain?
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
	let whisper_factory = RpcFactory { handle: whisper_network_handler, manager: whisper_filter_manager };
	let mut rpc_handler : jsonrpc_core::MetaIoHandler<WhisperMetadata, _> = jsonrpc_core::MetaIoHandler::default();

	let network = Arc::new(network);
	rpc_handler.extend_with(::parity_whisper::rpc::Whisper::to_delegate(whisper_factory.make_handler(network.clone())));
	rpc_handler.extend_with(::parity_whisper::rpc::WhisperPubSub::to_delegate(whisper_factory.make_handler(network.clone())));

	// -- 4) Launch RPC with handler
	let mut http_configuration = HttpConfiguration::default();
	let http_address = http_configuration.address().unwrap(); // .ok_or(some_error)?;
	let url = format!("{}:{}", http_address.0, http_address.1);
	let addr = url.parse().map_err(|_| format!("Invalid listen host/port given: {}", url));

	// let mut allowed_hosts: Option<Vec<Host>> = http_configuration.hosts.into();
	// allowed_hosts.as_mut().map(|mut hosts| {
	// 	hosts.push(format!("http://*.{}:*", DAPPS_DOMAIN).into());
	// 	hosts.push(format!("http://*.{}", DAPPS_DOMAIN).into());
	// });

	let threads = 1;
	let server = minihttp::ServerBuilder::new(rpc_handler)
				.threads(threads) // config param I guess // todo httpconfiguration
				// .meta_extractor(http_common::MiniMetaExtractor::new(extractor))
				// .cors(http_configuration.cors.into()) // cli
				//.allowed_hosts(allowed_hosts.into()) // cli
				.start_http(&addr.unwrap())
				.expect("Unable to start RPC server");
				// .map(HttpServer::Mini)?;

	println!("Starting listening...");
	server.wait().unwrap();

	Ok("OK".to_owned())
}
