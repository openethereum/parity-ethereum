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
//! Spawns an Ethereum network instance and attaches the Whisper protocol RPCs to it.
//!

extern crate ethcore_network_devp2p as devp2p;
extern crate ethcore_network as net;
extern crate parity_whisper as whisper;
extern crate serde;
extern crate docopt;
extern crate panic_hook;

extern crate jsonrpc_core;
extern crate jsonrpc_pubsub;
extern crate jsonrpc_http_server;

#[cfg(test)]
extern crate ethcore_devtools;

#[cfg(test)]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use std::sync::Arc;
use std::{fmt, io, process, env};
use jsonrpc_core::{Metadata, MetaIoHandler};
use jsonrpc_pubsub::{PubSubMetadata, Session};

const POOL_UNIT: usize = 1024 * 1024;
const USAGE: &'static str = r#"
Whisper CLI.
	Copyright 2017 Parity Technologies (UK) Ltd

Usage:
	whisper [options]
	whisper [-h | --help]

Options:
	--whisper-pool-size SIZE       Specify Whisper pool size [default: 10].
	-p, --port PORT                Specify which port to use [default: 8545].
	-a, --address ADDRESS          Specify which address to use [default: 127.0.0.1].
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
	Network(net::Error),
	SockAddr(std::net::AddrParseError),
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

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::SockAddr(ref e) => write!(f, "{}", e),
			Error::Docopt(ref e) => write!(f, "{}", e),
			Error::Io(ref e) => write!(f, "{}", e),
			Error::JsonRpc(ref e) => write!(f, "{:?}", e),
			Error::Network(ref e) => write!(f, "{:?}", e),
		}
	}
}

fn main() {
	panic_hook::set();

	match execute(env::args()) {
		Ok(_) => println!("ok"),
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

	// Filter manager that will dispatch `decryption tasks`
	// This provides the `Whisper` trait with all rpcs methods
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
		.start_http(&url.parse()?)?;

	server.wait();

	// This will never return
	Ok(())
}

#[cfg(test)]
mod test {
	use super::execute;
	use ethcore_devtools::http_client;
	use std::thread;
	use std::net::SocketAddr;
	use serde_json;
	use whisper::rpc::crypto;


	const UNIQUE_ID_SIZE_HEXSTRING: usize = 66;
	const PUBLIC_KEY_SIZE_HEXSTRING: usize = 130;
	const PRIVATE_KEY_SIZE_HEXSTRING: usize = 66;
	const SYMMETRIC_KEY_SIZE_HEXSTRING: usize = 66;

	#[derive(Debug, Deserialize)]
	struct JsonRpcResponse {
		id: usize,
		result: serde_json::Value,
		jsonrpc: String,
	}

	fn request<'a>(address: &SocketAddr, request: &'a str) -> http_client::Response {
		http_client::request(
			address,
			&format!("\
				POST / HTTP/1.1\r\n\
				Host: {}\r\n\
				Content-Type: application/json\r\n\
				Content-Length: {}\r\n\
				Connection: close\r\n\
				\r\n\
				{}",
				address, request.len(), request)
		)
	}

	fn parse_json<'a>(body: &'a str) -> JsonRpcResponse {
		let filter: String = body
			.split_whitespace()
			.filter(|c| c.starts_with("{"))
			.collect();
		serde_json::from_str(&filter).unwrap()
	}

	#[test]
	fn generate_keypair_and_post_message() {
		let address = &"127.0.0.1:8545".parse().unwrap();

		thread::spawn(move || {
			let command = vec!["whisper-cli", "-p", "8545"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

			execute(command).unwrap();
		});

		let unique_id = {
			let req = r#"{
				"method":"shh_newKeyPair",
				"params":[],
				"jsonrpc":"2.0",
				"id":1
			}"#;
			println!("req: {}", req);
			let response = request(address, req);
			let response_json = parse_json(&response.body);
			response_json.result
		};

		assert_eq!(unique_id.as_str().unwrap().len(), UNIQUE_ID_SIZE_HEXSTRING);

		let post_message = {
			let req = r#"{
				"method":"shh_post",
				"params":[{
					"from":"#.to_owned() + format!("{}", unique_id).as_ref() + r#",
					"topics":["0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6"],
					"payload":"0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6",
					"priority":40,
					"ttl":400
				}],
				"jsonrpc":"2.0",
				"id": 2
			}"#;
			let response = request(address, &req);
			let response_json = parse_json(&response.body);
			response_json.result
		};

		assert_eq!(post_message, true);
	}

	#[test]
	fn generate_keypair_and_delete() {
		let address = &"127.0.0.1:8546".parse().unwrap();

		thread::spawn(move || {
			let command = vec!["whisper-cli", "-p", "8546"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

			execute(command).unwrap();
		});


		let unique_id = {
			let req = r#"{
				"method":"shh_newKeyPair",
				"params":[],
				"jsonrpc":"2.0",
				"id":1
			}"#;
			let response = request(address, req);
			let response_json = parse_json(&response.body);
			response_json.result
		};
		assert_eq!(unique_id.as_str().unwrap().len(), UNIQUE_ID_SIZE_HEXSTRING);

		let public_key = {
			let req = r#"{
				"method": "shh_getPublicKey",
				"params": ["#.to_owned() + format!("{}", unique_id).as_ref() + r#"],
				"jsonrpc": "2.0",
				"id": 2
			}"#;
			let response = request(address, &req);
			let response_json = parse_json(&response.body);
			response_json.result
		};

		assert_eq!(public_key.as_str().unwrap().len(), PUBLIC_KEY_SIZE_HEXSTRING);

		let private_key = {
			let req = r#"{
				"method": "shh_getPrivateKey",
				"params": ["#.to_owned() + format!("{}", unique_id).as_ref() + r#"],
				"jsonrpc": "2.0",
				"id": 2
			}"#;
			let response = request(address, &req);
			let response_json = parse_json(&response.body);
			response_json.result
		};

		assert_eq!(private_key.as_str().unwrap().len(), PRIVATE_KEY_SIZE_HEXSTRING);
		println!("private_key: {}", private_key);

		let is_deleted = {
			let req = r#"{
				"method": "shh_deleteKey",
				"params": ["#.to_owned() + format!("{}", unique_id).as_ref() + r#"],
				"jsonrpc": "2.0",
				"id": 2
			}"#;
			let response = request(address, &req);
			let response_json = parse_json(&response.body);
			response_json.result
		};

		assert_eq!(is_deleted, true);
	}

	#[test]
	fn generate_symmetric_key_and_delete() {
		let address = &"127.0.0.1:8547".parse().unwrap();

		thread::spawn(move || {
			let command = vec!["whisper-cli", "-p", "8547"]
			.into_iter()
			.map(Into::into)
			.collect::<Vec<String>>();

			execute(command).unwrap();
		});


		let unique_id = {
			let req = r#"{
				"method":"shh_newSymKey",
				"params":[],
				"jsonrpc":"2.0",
				"id":1
			}"#;
			println!("req: {:?}", req);
			let response = request(address, req);
			println!("res: {:?}", response.body);
			let response_json = parse_json(&response.body);
			response_json.result
		};

		assert_eq!(unique_id.as_str().unwrap().len(), UNIQUE_ID_SIZE_HEXSTRING);

		let key = {
			let req = r#"{
				"method":"shh_getSymKey",
				"params":["#.to_owned() + &format!("{}", unique_id) + r#"],
				"jsonrpc":"2.0",
				"id": 2
			}"#;
			let response = request(address, &req);
			let response_json = parse_json(&response.body);
			response_json.result
		};

		assert_eq!(key.as_str().unwrap().len(), SYMMETRIC_KEY_SIZE_HEXSTRING);

		let is_deleted = {
			let req = r#"{
				"method": "shh_deleteKey",
				"params": ["#.to_owned() + format!("{}", unique_id).as_ref() + r#"],
				"jsonrpc": "2.0",
				"id": 2
			}"#;
			let response = request(address, &req);
			let response_json = parse_json(&response.body);
			response_json.result
		};

		assert_eq!(is_deleted, true);
	}

	#[test]
	fn message_filter() {


	}
}
