// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Whisper command line interface
//!
//! Spawns an Ethereum network instance and attaches the Whisper protocol RPCs to it.
//!

extern crate docopt;
extern crate ethcore_network_devp2p as devp2p;
extern crate ethcore_network as net;
extern crate parity_whisper as whisper;
extern crate serde;
extern crate panic_hook;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_http_server;
extern crate ethcore_logger as log;

#[macro_use]
extern crate log as rlog;

#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use std::{fmt, io, process, env, sync::Arc};
use jsonrpc_core::{Metadata, MetaIoHandler};
use jsonrpc_pubsub::{PubSubMetadata, Session};
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation};

const POOL_UNIT: usize = 1024 * 1024;
const USAGE: &'static str = r#"
Whisper CLI.
	Copyright 2017 Parity Technologies (UK) Ltd

Usage:
	whisper [options]
	whisper [-h | --help]

Options:
	--whisper-pool-size SIZE       Specify Whisper pool size [default: 10].
	-p, --port PORT                Specify which RPC port to use [default: 8545].
	-a, --address ADDRESS          Specify which address to use [default: 127.0.0.1].
	-l, --log LEVEL				   Specify the logging level. Must conform to the same format as RUST_LOG [default: Error].
	-h, --help                     Display this message and exit.
"#;

#[derive(Clone, Default)]
struct Meta;

impl Metadata for Meta {}

impl PubSubMetadata for Meta {
	fn session(&self) -> Option<Arc<Session>> {
		None
	}
}

#[derive(Debug, Deserialize)]
struct Args {
	flag_whisper_pool_size: usize,
	flag_port: String,
	flag_address: String,
	flag_log: String,
}

struct WhisperPoolHandle {
	/// Pool handle.
	handle: Arc<whisper::net::Network<Arc<whisper::rpc::FilterManager>>>,
	/// Network manager.
	net: Arc<devp2p::NetworkService>,
}

impl whisper::rpc::PoolHandle for WhisperPoolHandle {
	fn relay(&self, message: whisper::message::Message) -> bool {
		let mut res = false;
		let mut message = Some(message);
		self.with_proto_context(whisper::net::PROTOCOL_ID, &mut |ctx| {
			if let Some(message) = message.take() {
				res = self.handle.post_message(message, ctx);
			}
		});
		res
	}

	fn pool_status(&self) -> whisper::net::PoolStatus {
		self.handle.pool_status()
	}
}

impl WhisperPoolHandle {
	fn with_proto_context(&self, proto: net::ProtocolId, f: &mut FnMut(&net::NetworkContext)) {
		self.net.with_context_eval(proto, f);
	}
}

struct RpcFactory {
	handle: Arc<whisper::Network<Arc<whisper::rpc::FilterManager>>>,
	manager: Arc<whisper::rpc::FilterManager>,
}

impl RpcFactory {
	fn make_handler(&self, net: Arc<devp2p::NetworkService>) -> whisper::rpc::WhisperClient<WhisperPoolHandle, Meta> {
		let whisper_pool_handle = WhisperPoolHandle { handle: self.handle.clone(), net: net };
		whisper::rpc::WhisperClient::new(whisper_pool_handle, self.manager.clone())
	}
}

#[derive(Debug)]
enum Error {
	Docopt(docopt::Error),
	Io(io::Error),
	JsonRpc(jsonrpc_core::Error),
	Network(net::Error),
	SockAddr(std::net::AddrParseError),
	Logger(String),
}

impl From<std::net::AddrParseError> for Error {
	fn from(err: std::net::AddrParseError) -> Self {
		Error::SockAddr(err)
	}
}

impl From<net::Error> for Error {
	fn from(err: net::Error) -> Self {
		Error::Network(err)
	}
}

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

impl From<jsonrpc_core::Error> for Error {
	fn from(err: jsonrpc_core::Error) -> Self {
		Error::JsonRpc(err)
	}
}

impl From<String> for Error {
	fn from(err: String) -> Self {
		Error::Logger(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::SockAddr(ref e) => write!(f, "SockAddrError: {}", e),
			Error::Docopt(ref e) => write!(f, "DocoptError: {}", e),
			Error::Io(ref e) => write!(f, "IoError: {}", e),
			Error::JsonRpc(ref e) => write!(f, "JsonRpcError: {:?}", e),
			Error::Network(ref e) => write!(f, "NetworkError: {}", e),
			Error::Logger(ref e) => write!(f, "LoggerError: {}", e),
		}
	}
}

fn main() {
	panic_hook::set();

	match execute(env::args()) {
		Ok(_) => {
			println!("whisper-cli terminated");
			process::exit(1);
		}
		Err(err) => {
			println!("{}", err);
			process::exit(1);
		},
	}
}

fn execute<S, I>(command: I) -> Result<(), Error> where I: IntoIterator<Item=S>, S: AsRef<str> {

	// Parse arguments
	let args: Args = Docopt::new(USAGE).and_then(|d| d.argv(command).deserialize())?;
	let pool_size = args.flag_whisper_pool_size * POOL_UNIT;
	let url = format!("{}:{}", args.flag_address, args.flag_port);

	initialize_logger(args.flag_log)?;
	info!(target: "whisper-cli", "start");

	// Filter manager that will dispatch `decryption tasks`
	let manager = Arc::new(whisper::rpc::FilterManager::new()?);

	// Whisper protocol network handler
	let whisper_network_handler = Arc::new(whisper::net::Network::new(pool_size, manager.clone()));

	// Create network service
	let network = devp2p::NetworkService::new(net::NetworkConfiguration::new_local(), None)?;

	// Start network service
	network.start()?;

	// Attach whisper protocol to the network service
	network.register_protocol(whisper_network_handler.clone(), whisper::net::PROTOCOL_ID, whisper::net::PACKET_COUNT,
							  whisper::net::SUPPORTED_VERSIONS)?;
	network.register_protocol(Arc::new(whisper::net::ParityExtensions), whisper::net::PARITY_PROTOCOL_ID,
							  whisper::net::PACKET_COUNT, whisper::net::SUPPORTED_VERSIONS)?;

	// Request handler
	let mut io = MetaIoHandler::default();

	// Shared network service
	let shared_network = Arc::new(network);

	// Pool handler
	let whisper_factory = RpcFactory { handle: whisper_network_handler, manager: manager };

	io.extend_with(whisper::rpc::Whisper::to_delegate(whisper_factory.make_handler(shared_network.clone())));
	io.extend_with(whisper::rpc::WhisperPubSub::to_delegate(whisper_factory.make_handler(shared_network.clone())));

	let server = jsonrpc_http_server::ServerBuilder::new(io)
		.cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Null]))
		.start_http(&url.parse()?)?;

	server.wait();

	// This will never return if the http server runs without errors
	Ok(())
}

fn initialize_logger(log_level: String) -> Result<(), String> {
	let mut l = log::Config::default();
	l.mode = Some(log_level);
	log::setup_log(&l)?;
	Ok(())
}


#[cfg(test)]
mod tests {
	use super::execute;

	#[test]
	fn invalid_argument() {
		let command = vec!["whisper-cli", "--foo=12"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		assert!(execute(command).is_err());
	}

	#[test]
	#[ignore]
	fn privileged_port() {
		let command = vec!["whisper-cli", "--port=3"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		assert!(execute(command).is_err());
	}

	#[test]
	fn invalid_ip_address() {
		let command = vec!["whisper-cli", "--address=x.x.x.x"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		assert!(execute(command).is_err());
	}

	#[test]
	fn invalid_whisper_pool_size() {
		let command = vec!["whisper-cli", "--whisper-pool-size=-100000000000000000000000000000000000000"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		assert!(execute(command).is_err());
	}
}
