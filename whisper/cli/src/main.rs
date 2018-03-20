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

//! Whisper command line interface
//!
//! Connects to the Ethereum network and allows to transmit secure messages
//!
//! Questions that I need to understand:
//!

extern crate ethcore_network_devp2p as devp2p;
extern crate ethcore_network as net;
extern crate parity_whisper as whisper;
extern crate serde;
extern crate docopt;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_http_server;

#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use std::sync::Arc;
use std::{fmt, io, process, env};
use jsonrpc_core::{Metadata, MetaIoHandler};
use jsonrpc_pubsub::{PubSubMetadata, Session};

const URL: &'static str = "127.0.0.1:8545";

const USAGE: &'static str = r#"
Whisper.
    Copyright 2017 Parity Technologies (UK) Ltd
Usage:
    whisper [options]
    whisper [-h | --help]
Options:
    --whisper-pool-size SIZE       Specify Whisper pool size [default: 10].
    -h, --help                     Display this message and exit.
"#;

// Dummy
#[derive(Clone, Default)]
struct Meta(usize);
impl Metadata for Meta {}
impl PubSubMetadata for Meta {
	fn session(&self) -> Option<Arc<Session>> {
		unimplemented!();
	}
}

#[derive(Debug, Deserialize)]
struct Args {
	flag_whisper_pool_size: usize,
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
	pub fn make_handler(&self, net: Arc<devp2p::NetworkService>) -> whisper::rpc::WhisperClient<WhisperPoolHandle, Meta> {
		let whisper_pool_handle = WhisperPoolHandle { handle: self.handle.clone(), net: net };
		whisper::rpc::WhisperClient::new(whisper_pool_handle, self.manager.clone())
	}
}

#[derive(Debug)]
enum Error {
	Docopt(docopt::Error),
	Io(io::Error),
	JsonRpc(jsonrpc_core::Error),
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

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::Docopt(ref e) => write!(f, "{}", e),
			Error::Io(ref e) => write!(f, "{}", e),
			Error::JsonRpc(ref e) => write!(f, "{:?}", e),
		}
	}
}

fn main() {
	match execute(env::args()) {
		Ok(ok) => println!("{}", ok),
		Err(err) => {
			println!("{}", err);
			process::exit(1);
		},
	}
}

fn execute<S, I>(command: I) -> Result<String, Error> where I: IntoIterator<Item=S>, S: AsRef<str> {

	// Parse arguments
	let args: Args = Docopt::new(USAGE).and_then(|d| d.argv(command).deserialize())?;

	// Dummy this should be parsed from the args entered by the user
	let pool_size = 1000;

	// Filter manager that will dispatch `decryption tasks`
	// This provides the `Whisper` trait with all rpcs methods
	// FIXME: Filter kinds as arg
	let manager = Arc::new(whisper::rpc::FilterManager::new().unwrap());

	// Whisper protocol network handler
	let whisper_network_handler = Arc::new(whisper::net::Network::new(pool_size, manager.clone()));

	// Create network service
	let network = devp2p::NetworkService::new(net::NetworkConfiguration::new_local(), None).expect("Error creating network service");

	// Start network service
	network.start().expect("Error starting service");

	// Attach whisper protocol to the network service
	network.register_protocol(whisper_network_handler.clone(), whisper::net::PROTOCOL_ID, whisper::net::PACKET_COUNT,
							  whisper::net::SUPPORTED_VERSIONS).unwrap();
	network.register_protocol(Arc::new(whisper::net::ParityExtensions), whisper::net::PARITY_PROTOCOL_ID,
							  whisper::net::PACKET_COUNT, whisper::net::SUPPORTED_VERSIONS).unwrap();

	// Request handler
	let mut io = MetaIoHandler::default();

	// Shared network service
	let shared_network = Arc::new(network);

	// Pool handler
	let whisper_factory = RpcFactory { handle: whisper_network_handler, manager: manager };

	io.extend_with(whisper::rpc::Whisper::to_delegate(whisper_factory.make_handler(shared_network.clone())));
	io.extend_with(whisper::rpc::WhisperPubSub::to_delegate(whisper_factory.make_handler(shared_network.clone())));

	let server = jsonrpc_http_server::ServerBuilder::new(io)
		.start_http(&URL.parse().unwrap())
		.expect("Unable to start server");

	server.wait();

	Ok("foo".into())
}






