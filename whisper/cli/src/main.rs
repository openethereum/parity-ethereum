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
extern crate parity_rpc;
extern crate ethcore_network as net;
extern crate ethcore_ipc_hypervisor as hypervisor;

use std::str::FromStr;


extern crate jsonrpc_core;
extern crate jsonrpc_http_server as http;
extern crate jsonrpc_ipc_server as ipc;
extern crate jsonrpc_minihttp_server as minihttp;
extern crate jsonrpc_pubsub;
extern crate parity_dapps;
extern crate ethsync;

// #[cfg(feature="ipc")]
pub mod service_urls {
	use std::path::PathBuf;

	pub const CLIENT: &'static str = "parity-chain.ipc";
	pub const SNAPSHOT: &'static str = "parity-snapshot.ipc";
	pub const SYNC: &'static str = "parity-sync.ipc";
	pub const SYNC_NOTIFY: &'static str = "parity-sync-notify.ipc";
	pub const NETWORK_MANAGER: &'static str = "parity-manage-net.ipc";
	pub const SYNC_CONTROL: &'static str = "parity-sync-control.ipc";
	pub const LIGHT_PROVIDER: &'static str = "parity-light-provider.ipc";

	#[cfg(feature="stratum")]
	pub const STRATUM_CONTROL: &'static str = "parity-stratum-control.ipc";

	pub fn with_base(data_dir: &str, service_path: &str) -> String {
		let mut path = PathBuf::from(data_dir);
		path.push(service_path);

		format!("ipc://{}", path.to_str().unwrap())
	}
}
use std::path::Path;
use hypervisor::Hypervisor;
use hypervisor::{SYNC_MODULE_ID, BootArgs, HYPERVISOR_IPC_URL};
// #[cfg(feature="ipc")]
pub fn hypervisor(base_path: &Path) -> Option<Hypervisor> {
	Some(Hypervisor
		::with_url(&service_urls::with_base(base_path.to_str().unwrap(), HYPERVISOR_IPC_URL))
		.io_path(base_path.to_str().unwrap()))
}

/// RPC dependencies can be used to initialize RPC endpoints from APIs.
// pub trait Dependencies {
// 	type Notifier: ActivityNotifier;

// 	/// Create the activity notifier.
// 	fn activity_notifier(&self) -> Self::Notifier;

// 	/// Extend the given I/O handler with endpoints for each API.
// 	fn extend_with_set<S>(
// 		&self,
// 		handler: &mut jsonrpc_core::MetaIoHandler<Metadata, S>,
// 		apis: &HashSet<Api>,
// 	) where S: jsonrpc_core::Middleware<Metadata>;
// }


/// RPC dependencies can be used to initialize RPC endpoints from APIs.
// pub trait RpcApisDependencies {
// 	type Notifier: ActivityNotifier;

// 	/// Create the activity notifier.
// 	fn activity_notifier(&self) -> Self::Notifier;

// 	/// Extend the given I/O handler with endpoints for each API.
// 	fn extend_with_set<S>(
// 		&self,
// 		handler: &mut jsonrpc_core::MetaIoHandler<Metadata, S>,
// 		apis: &HashSet<Api>,
// 	) where S: jsonrpc_core::Middleware<Metadata>;
// }

// pub struct Dependencies<D: RpcApisDependencies> {
// 	pub apis: Arc<D>,
// 	pub remote: tokio_core::reactor::Remote,
// 	pub stats: Arc<RpcStats>,
// 	pub pool: Option<CpuPool>,
// }

extern crate ethcore_ipc_nano as nanoipc;
use nanoipc::{GuardedSocket, NanoSocket, generic_client, fast_client};
use parity_dapps as dapps;
use parity_rpc::informant::{RpcStats, Middleware, CpuPool, ActivityNotifier};
use parity_rpc::Metadata;
use std::collections::HashSet;
use std::{env, fmt, process};
use docopt::Docopt;
use std::io;
use net::*;
use std::net::SocketAddr;
// use http::tokio_core;
use http::{
	hyper,
	tokio_core,
	RequestMiddleware, RequestMiddlewareAction,
	AccessControlAllowOrigin, Host, DomainsValidation
};

const DAPPS_DOMAIN: &'static str = "web3.site";

mod http_common {
/* start http_common.rs */

use jsonrpc_core;
use jsonrpc_core::MetaIoHandler;
use http;
use hyper;
use minihttp;
// use parity_reactor::TokioRemote;
use tokio_core::reactor::{Remote as TokioRemote};


/// HTTP RPC server impl-independent metadata extractor
pub trait HttpMetaExtractor: Send + Sync + 'static {
	/// Type of Metadata
	type Metadata: jsonrpc_core::Metadata;
	/// Extracts metadata from given params.
	fn read_metadata(&self, origin: Option<String>, user_agent: Option<String>, dapps_origin: Option<String>) -> Self::Metadata;
}

pub struct HyperMetaExtractor<T> {
	extractor: T,
}

impl<T> HyperMetaExtractor<T> {
	pub fn new(extractor: T) -> Self {
		HyperMetaExtractor {
			extractor: extractor,
		}
	}
}

impl<M, T> http::MetaExtractor<M> for HyperMetaExtractor<T> where
	T: HttpMetaExtractor<Metadata = M>,
	M: jsonrpc_core::Metadata,
{
	fn read_metadata(&self, req: &hyper::server::Request<hyper::net::HttpStream>) -> M {
		let as_string = |header: Option<&http::request_response::header::Raw>| header
			.and_then(|raw| raw.one())
			.map(|raw| String::from_utf8_lossy(raw).into_owned());

		let origin = as_string(req.headers().get_raw("origin"));
		let user_agent = as_string(req.headers().get_raw("user-agent"));
		let dapps_origin = as_string(req.headers().get_raw("x-parity-origin"));
		self.extractor.read_metadata(origin, user_agent, dapps_origin)
	}
}

pub struct MiniMetaExtractor<T> {
	extractor: T,
}

impl<T> MiniMetaExtractor<T> {
	pub fn new(extractor: T) -> Self {
		MiniMetaExtractor {
			extractor: extractor,
		}
	}
}

impl<M, T> minihttp::MetaExtractor<M> for MiniMetaExtractor<T> where
	T: HttpMetaExtractor<Metadata = M>,
	M: jsonrpc_core::Metadata,
{
	fn read_metadata(&self, req: &minihttp::Req) -> M {
		let origin = req.header("origin").map(|h| h.to_owned());
		let user_agent = req.header("user-agent").map(|h| h.to_owned());
		let dapps_origin = req.header("x-parity-origin").map(|h| h.to_owned());

		self.extractor.read_metadata(origin, user_agent, dapps_origin)
	}
}

/* end http_common.rs */
}


use std::sync::Arc; // no
use parity_whisper::net::{self as whisper_net, Network as WhisperNetwork};
use parity_whisper::rpc::{WhisperClient, FilterManager, PoolHandle};
use parity_whisper::message::Message;

/// Standard pool handle. {from parity/whisper}
pub struct NetworkPoolHandle {
	/// Pool handle.
	handle: Arc<WhisperNetwork<Arc<FilterManager>>>,
	/// Network manager.
	net: Arc<NetworkService>,
}

// impl PoolHandle for NetworkPoolHandle {
// 	fn relay(&self, message: Message) -> bool {
// 		let mut res = false;
// 		let mut message = Some(message);
// 		self.net.with_proto_context(whisper_net::PROTOCOL_ID, &mut move |ctx| {
// 			if let Some(message) = message.take() {
// 				res = self.handle.post_message(message, ctx);
// 			}
// 		});
// 		res
// 	}

// 	fn pool_status(&self) -> whisper_net::PoolStatus {
// 		self.handle.pool_status()
// 	}
// }

/// Factory for standard whisper RPC. {from parity/whisper}
// pub struct RpcFactory {
// 	net: Arc<WhisperNetwork<Arc<FilterManager>>>,
// 	manager: Arc<FilterManager>,
// }

// impl RpcFactory {
// 	pub fn make_handler(&self, net: Arc<ManageNetwork>) -> WhisperClient<NetPoolHandle, Metadata> {
// 		let handle = NetPoolHandle { handle: self.net.clone(), net: net };
// 		WhisperClient::new(handle, self.manager.clone())
// 	}
// }

use ethsync::ManageNetwork; // bad
// use ethsync::api::{SyncClient, NetworkManagerClient};

// use parity_rpc::*;


/// HTTP server implementation-specific settings.
pub enum HttpSettings<R: RequestMiddleware> {
	/// Enable fast minihttp server with given number of threads.
	Threads(usize),
	/// Enable standard server with optional dapps middleware.
	Dapps(Option<R>),
}


/// RPC HTTP Server instance
pub enum HttpServer {
	/// Fast MiniHTTP variant
	Mini(minihttp::Server),
	/// Hyper variant
	Hyper(http::Server),
}


/// RPC HTTP Server error
#[derive(Debug)]
pub enum HttpServerError {
	/// IO error
	Io(::std::io::Error),
	/// Other hyper error
	Hyper(hyper::Error),
}

impl HttpServer {
	/// Returns current listening address.
	pub fn address(&self) -> &SocketAddr {
		match *self {
			HttpServer::Mini(ref s) => s.address(),
			HttpServer::Hyper(ref s) => &s.addrs()[0],
		}
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

// pub use ethsync::api::{NetworkManagerClient};

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

// go to clap

/*


Commands:
    generate           Generates new ethereum key.
    random             Random generation.
    prefix             Random generation, but address must start with a prefix
    brain              Generate new key from string seed.
    sign               Sign message using secret.
    verify             Verify signer of the signature.
	*/

#[derive(Debug, Deserialize)]
struct Args {
	flag_whisper_pool_size: usize,
}

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

	// Questions :
	// - No need for Arc<>, right? Pas besoin d'envoyer à travers des threads, right ?

	// Todo :
	// - Réorganiser en structs pour fermer les dépendances

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
	// let whisper_factory = RpcFactory { net: whisper_network_handler, manager: whisper_filter_manager };
	// let whisper_rpc_handler = whisper_factory.make_handler(Arc::new(network));

	let handle2 = NetworkPoolHandle { handle: whisper_network_handler.clone(), net: Arc::new(network) };

//	let handle = NetPoolHandle { handle: whisper_network_handler.clone(), net: Arc::new(network) };
	// so, whisperclient::new needs a PoolHandle
	// so: netpoolhandle needs a ManageNetwork (or not ,actually)
	// if I want to make 'network' a ManageNetwork, I need to impl it. but I would implement an external trait to an external struct, so that wouldn't work.


/*
hey Rob, WhiperClient::new expects a PoolHandle with a struct implementing ManageNetwork, however NetworkService::new doesn't implement ManageNetwork by default. should I implement ManageNetwork for NetworkService::new in the whisper cli file? (if so, I imagine I'd have to use a wrapper of sorts, as both the trait and the struct are external to the parity cli ?)
*/
	let whisper_rpc_handler : WhisperClient<NetworkPoolHandle, Metadata> = WhisperClient::new(handle2, whisper_filter_manager.clone());

	let mut rpc_handler : jsonrpc_core::MetaIoHandler<Metadata, _> = jsonrpc_core::MetaIoHandler::default(); // ou IoHandler::new();
	// rpc_handler.extend_with(::parity_whisper::rpc::Whisper::to_delegate(whisper_rpc_handler));

	// -- 4) Launch RPC with handler

	// Api::WhisperPubSub
	// if !for_generic_pubsub {
		// needs arc managenetwork
	// handler.extend_with(
	// 	::parity_whisper::rpc::WhisperPubSub::to_delegate(whisper_pubsub_handler) // not sure why
	// );
	// }

	// STart whisper service
	// Create RPC handler with the service handle
	// Whisper => RPC
	// (or 2 and 3 reversed)
	// make wrapper around devP2P and whisper; make it implement PoolHandle
	// struct P2P, whisper
	// poolhandle parity CLI can copy
	// don't need managernetwork

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
	minihttp::ServerBuilder::new(rpc_handler) // yay handler => rpc
				.threads(threads) // config param I guess // todo httpconfiguration
				// .meta_extractor(http_common::MiniMetaExtractor::new(extractor))
				// .cors(http_configuration.cors.into()) // cli
				//.allowed_hosts(allowed_hosts.into()) // cli
				.start_http(&addr.unwrap())
				.map(HttpServer::Mini)?;

	println!("Rpc server listening.");




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
