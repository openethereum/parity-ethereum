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
//!		* WhisperPool (is it a pool a instances that can "whisper"?)
//!		* Communication model: PublishSubscribe/jsonhttp server
//!		* Use built trait by Whisper?
//!		* MetaIoHandler?
//!		* Implement Own Meta data
//!		* Should communication port be an arg to the CLI?
//!		*
//!

extern crate ethcore_network_devp2p as devp2p;
extern crate ethcore_network as net;
extern crate parity_whisper as whisper;
extern crate serde;
extern crate docopt;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_minihttp_server as minihttp;

#[macro_use]
extern crate jsonrpc_macros;
#[macro_use]
extern crate serde_derive;

use net::*;
use docopt::Docopt;
use std::sync::Arc;
use std::{fmt, io, process, env};
use minihttp::{cors, ServerBuilder, DomainsValidation, Req};

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

#[derive(Debug, Deserialize)]
struct Args {
	flag_whisper_pool_size: usize,
}

pub struct WhisperPoolHandle {
	/// Pool handle.
	handle: Arc<whisper::net::Network<Arc<whisper::rpc::FilterManager>>>,
	/// Network manager.
	net: Arc<devp2p::NetworkService>,
}

impl whisper::rpc::PoolHandle for WhisperPoolHandle {
	fn relay(&self, message: whisper::Message) -> bool {
		unimplemented!();
	}

	fn pool_status(&self) -> whisper::net::PoolStatus {
		self.handle.pool_status()
	}
}
//
impl WhisperPoolHandle {
	fn with_proto_context(&self, proto: net::ProtocolId, f: &mut FnMut(&devp2p::NetworkContext)) {
		unimplemented!();
		// self.net.with_context_eval(proto, f);
	}
}
//
pub struct RpcFactory {
	handle: Arc<whisper::Network<Arc<whisper::rpc::FilterManager>>>,
	manager: Arc<whisper::rpc::FilterManager>,
}
//
impl RpcFactory {
	pub fn make_handler(&self, net: Arc<devp2p::NetworkService>) -> whisper::rpc::WhisperClient<WhisperPoolHandle, whisper::rpc::Meta> {
		let whisper_pool_handle = WhisperPoolHandle { handle: self.handle.clone(), net: net };
		let whisper_rpc_handler : whisper::rpc::WhisperClient<WhisperPoolHandle, whisper::rpc::Meta> = whisper::rpc::WhisperClient::new(whisper_pool_handle, self.manager.clone());
		whisper_rpc_handler
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

	let pool_size = args.flag_whisper_pool_size;

	// Create Whisper N/W
	let manager = Arc::new(whisper::rpc::FilterManager::new().unwrap());
	let whisper_network_handler = Arc::new(whisper::net::Network::new(pool_size, manager.clone()));

	// Instantiate Whisper network and attach to it the network handler
	let network = devp2p::NetworkService::new(NetworkConfiguration::new_local(), None).expect("Error creating network service");
	network.start().expect("Error starting service");
	network.register_protocol(whisper_network_handler.clone(), whisper::net::PROTOCOL_ID, whisper::net::PACKET_COUNT, whisper::net::SUPPORTED_VERSIONS).unwrap();
	network.register_protocol(Arc::new(whisper::net::ParityExtensions), whisper::net::PARITY_PROTOCOL_ID, whisper::net::PACKET_COUNT, whisper::net::SUPPORTED_VERSIONS).unwrap();

	// request handler
	let mut io: jsonrpc_core::MetaIoHandler<whisper::rpc::Meta, _> = jsonrpc_core::MetaIoHandler::default();
	let rpc = RpcFactory { handle: whisper_network_handler, manager: manager };


	let n = Arc::new(network);
	io.extend_with(whisper::rpc::Whisper::to_delegate(rpc.make_handler(n.clone())));
	io.extend_with(whisper::rpc::WhisperPubSub::to_delegate(rpc.make_handler(n.clone())));


	let server = ServerBuilder::new(io)
		.meta_extractor(|req: &Req| {
			whisper::rpc::Meta(req.header("Origin").map(|v| v.len()).unwrap_or_default())
		})
		.threads(1)
		.start_http(&"127.0.0.1:3030".parse().unwrap())
		.expect("Unable to start RPC server");

	Ok("SUCCESS".to_string())
}








