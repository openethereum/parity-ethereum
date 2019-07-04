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

//! Whisper command line interface
//!
//! Spawns an Ethereum network instance and attaches the Whisper protocol RPCs to it.
//!

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", deny(clippy, clippy_pedantic))]

extern crate docopt;
extern crate env_logger;
extern crate ethcore_network as net;
extern crate ethcore_network_devp2p as devp2p;
extern crate panic_hook;
extern crate parity_whisper as whisper;
extern crate serde;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_http_server;
extern crate ethkey;
extern crate rustc_hex;

#[macro_use]
extern crate log as rlog;

#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use std::{fmt, io, process, env, sync::Arc};
use jsonrpc_core::{Metadata, MetaIoHandler};
use jsonrpc_pubsub::{PubSubMetadata, Session};
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation};
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::str::FromStr;
use ethkey::Secret;
use rustc_hex::FromHex;

const POOL_UNIT: usize = 1024 * 1024;
const USAGE: &'static str = r#"
Parity Whisper-v2 CLI.
	Copyright 2015-2019 Parity Technologies (UK) Ltd.

Usage:
	whisper [options]
	whisper [-h | --help]

Options:
	--whisper-pool-size SIZE       Specify Whisper pool size [default: 10].
	-p, --port PORT                Specify which P2P port to use [default: random].
	-a, --address ADDRESS          Specify which P2P address to use [default: 127.0.0.1].
	-s, --secret KEYFILE           Specify which file contains the key to generate the enode.
	-P, --rpc-port PORT            Specify which RPC port to use [default: 8545].
	-A, --rpc-address ADDRESS      Specify which RPC address to use [default: 127.0.0.1].
	-l, --log LEVEL                Specify the logging level. Must conform to the same format as RUST_LOG [default: Error].
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
	flag_rpc_port: String,
	flag_rpc_address: String,
	flag_log: String,
	flag_secret: String,
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
	FromHex(rustc_hex::FromHexError),
	ParseInt(std::num::ParseIntError),
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

impl From<rustc_hex::FromHexError> for Error {
	fn from(err: rustc_hex::FromHexError) -> Self {
		Error::FromHex(err)
	}
}

impl From<std::num::ParseIntError> for Error {
	fn from(err: std::num::ParseIntError) -> Self {
		Error::ParseInt(err)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::SockAddr(ref e) => write!(f, "{}", e),
			Error::Docopt(ref e) => write!(f, "{}", e),
			Error::Io(ref e) => write!(f, "{}", e),
			Error::JsonRpc(ref e) => write!(f, "{:?}", e),
			Error::Network(ref e) => write!(f, "{}", e),
			Error::ParseInt(ref e) => write!(f, "Invalid port: {}", e),
			Error::FromHex(ref e) => write!(f, "Error deciphering key: {}", e),
		}
	}
}

fn main() {
	panic_hook::set_abort();

	match execute(env::args()) {
		Ok(_) => {
			println!("whisper-cli terminated");
			process::exit(1);
		},
		Err(Error::Docopt(ref e)) => e.exit(),
		Err(err) => {
			println!("{}", err);
			process::exit(1);
		}
	}
}

fn execute<S, I>(command: I) -> Result<(), Error> where I: IntoIterator<Item=S>, S: AsRef<str> {

	// Parse arguments
	let args: Args = Docopt::new(USAGE).and_then(|d| d.argv(command).deserialize())?;
	let pool_size = args.flag_whisper_pool_size * POOL_UNIT;
	let rpc_url = format!("{}:{}", args.flag_rpc_address, args.flag_rpc_port);

	initialize_logger(args.flag_log);
	info!(target: "whisper-cli", "start");

	// Filter manager that will dispatch `decryption tasks`
	let manager = Arc::new(whisper::rpc::FilterManager::new()?);

	// Whisper protocol network handler
	let whisper_network_handler = Arc::new(whisper::net::Network::new(pool_size, manager.clone()));

	let network_config = {
		let mut cfg = net::NetworkConfiguration::new();
		let port = match args.flag_port.as_str() {
			"random" => 0 as u16,
			port => port.parse::<u16>()?,

		};
		let addr = Ipv4Addr::from_str(&args.flag_address[..])?;
		cfg.listen_address = Some(SocketAddr::V4(SocketAddrV4::new(addr, port)));
		cfg.use_secret = match args.flag_secret.as_str() {
			"" => None,
			fname => {
				let key_text = std::fs::read_to_string(fname)?;
				let key : Vec<u8> = FromHex::from_hex(key_text.as_str())?;
				Secret::from_slice(key.as_slice())
			}
		};
		cfg.nat_enabled = false;
		cfg
	};

	// Create network service
	let network = devp2p::NetworkService::new(network_config, None)?;

	// Start network service
	network.start().map_err(|(err, _)| err)?;

	// Attach whisper protocol to the network service
	network.register_protocol(whisper_network_handler.clone(), whisper::net::PROTOCOL_ID,
							  whisper::net::SUPPORTED_VERSIONS)?;
	network.register_protocol(Arc::new(whisper::net::ParityExtensions), whisper::net::PARITY_PROTOCOL_ID,
							  whisper::net::SUPPORTED_VERSIONS)?;

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
		.start_http(&rpc_url.parse()?)?;

	server.wait();

	// This will never return if the http server runs without errors
	Ok(())
}

fn initialize_logger(log_level: String) {
	env_logger::Builder::from_env(env_logger::Env::default())
		.parse(&log_level)
		.init();
}

#[cfg(test)]
mod tests {
	use super::execute;

	#[test]
	fn invalid_argument() {
		let command = vec!["whisper", "--foo=12"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		assert!(execute(command).is_err());
	}

	#[test]
	#[ignore]
	fn privileged_port() {
		let command = vec!["whisper", "--port=3"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		assert!(execute(command).is_err());
	}

	#[test]
	fn invalid_ip_address() {
		let command = vec!["whisper", "--address=x.x.x.x"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		assert!(execute(command).is_err());
	}

	#[test]
	// The Whisper pool size is of type usize. Invalid Whisper pool sizes include
	// values below 0 and either above 2 ** 64 - 1 on a 64-bit processor or
	// above 2 ** 32 - 1 on a 32-bit processor.
	fn invalid_whisper_pool_size() {
		let command_pool_size_too_low = vec!["whisper", "--whisper-pool-size=-1"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		let command_pool_size_too_high = vec!["whisper", "--whisper-pool-size=18446744073709552000"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

		assert!(execute(command_pool_size_too_low).is_err());
		assert!(execute(command_pool_size_too_high).is_err());
	}
}
