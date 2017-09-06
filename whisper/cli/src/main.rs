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

use std::str::FromStr;

extern crate jsonrpc_core;
extern crate jsonrpc_http_server as http;
extern crate jsonrpc_ipc_server as ipc;
extern crate jsonrpc_minihttp_server as minihttp;
extern crate jsonrpc_pubsub;
extern crate parity_dapps;

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
pub trait RpcApisDependencies {
	type Notifier: ActivityNotifier;

	/// Create the activity notifier.
	fn activity_notifier(&self) -> Self::Notifier;

	/// Extend the given I/O handler with endpoints for each API.
	fn extend_with_set<S>(
		&self,
		handler: &mut jsonrpc_core::MetaIoHandler<Metadata, S>,
		apis: &HashSet<Api>,
	) where S: jsonrpc_core::Middleware<Metadata>;
}

pub struct Dependencies<D: RpcApisDependencies> {
	pub apis: Arc<D>,
	pub remote: tokio_core::reactor::Remote,
	pub stats: Arc<RpcStats>,
	pub pool: Option<CpuPool>,
}

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
use parity_whisper::rpc::{WhisperClient, FilterManager};

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


#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Api {
	/// Web3 (Safe)
	Web3,
	/// Net (Safe)
	Net,
	/// Eth (Safe)
	Eth,
	/// Eth Pub-Sub (Safe)
	EthPubSub,
	/// Geth-compatible "personal" API (DEPRECATED; only used in `--geth` mode.)
	Personal,
	/// Signer - Confirm transactions in Signer (UNSAFE: Passwords, List of transactions)
	Signer,
	/// Parity - Custom extensions (Safe)
	Parity,
	/// Parity PubSub - Generic Publish-Subscriber (Safety depends on other APIs exposed).
	ParityPubSub,
	/// Parity Accounts extensions (UNSAFE: Passwords, Side Effects (new account))
	ParityAccounts,
	/// Parity - Set methods (UNSAFE: Side Effects affecting node operation)
	ParitySet,
	/// Traces (Safe)
	Traces,
	/// Rpc (Safe)
	Rpc,
	/// SecretStore (Safe)
	SecretStore,
	/// Whisper (Safe)
	// TODO: _if_ someone guesses someone else's key or filter IDs they can remove
	// BUT these are all ephemeral so it seems fine.
	Whisper,
	/// Whisper Pub-Sub (Safe but same concerns as above).
	WhisperPubSub,
}

impl FromStr for Api {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use self::Api::*;

		match s {
			"web3" => Ok(Web3),
			"net" => Ok(Net),
			"eth" => Ok(Eth),
			"pubsub" => Ok(EthPubSub),
			"personal" => Ok(Personal),
			"signer" => Ok(Signer),
			"parity" => Ok(Parity),
			"parity_pubsub" => Ok(ParityPubSub),
			"parity_accounts" => Ok(ParityAccounts),
			"parity_set" => Ok(ParitySet),
			"traces" => Ok(Traces),
			"rpc" => Ok(Rpc),
			"secretstore" => Ok(SecretStore),
			"shh" => Ok(Whisper),
			"shh_pubsub" => Ok(WhisperPubSub),
			api => Err(format!("Unknown api: {}", api))
		}
	}
}

#[derive(Debug, Clone)]
pub enum ApiSet {
	// Safe context (like token-protected WS interface)
	SafeContext,
	// Unsafe context (like jsonrpc over http)
	UnsafeContext,
	// Public context (like public jsonrpc over http)
	PublicContext,
	// All possible APIs
	All,
	// Local "unsafe" context and accounts access
	IpcContext,
	// APIs for Parity Generic Pub-Sub
	PubSub,
	// Fixed list of APis
	List(HashSet<Api>),
}

impl Default for ApiSet {
	fn default() -> Self {
		ApiSet::UnsafeContext
	}
}

impl PartialEq for ApiSet {
	fn eq(&self, other: &Self) -> bool {
		self.list_apis() == other.list_apis()
	}
}

impl FromStr for ApiSet {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut apis = HashSet::new();

		for api in s.split(',') {
			match api {
				"all" => {
					apis.extend(ApiSet::All.list_apis());
				},
				"safe" => {
					// Safe APIs are those that are safe even in UnsafeContext.
					apis.extend(ApiSet::UnsafeContext.list_apis());
				},
				// Remove the API
				api if api.starts_with("-") => {
					let api = api[1..].parse()?;
					apis.remove(&api);
				},
				api => {
					let api = api.parse()?;
					apis.insert(api);
				},
			}
		}

		Ok(ApiSet::List(apis))
	}
}

impl ApiSet {
	/// Retains only APIs in given set.
	pub fn retain(self, set: Self) -> Self {
		ApiSet::List(&self.list_apis() & &set.list_apis())
	}

	pub fn list_apis(&self) -> HashSet<Api> {
		let mut public_list = [
			Api::Web3,
			Api::Net,
			Api::Eth,
			Api::EthPubSub,
			Api::Parity,
			Api::Rpc,
			Api::SecretStore,
			Api::Whisper,
			Api::WhisperPubSub,
		].into_iter().cloned().collect();

		match *self {
			ApiSet::List(ref apis) => apis.clone(),
			ApiSet::PublicContext => public_list,
			ApiSet::UnsafeContext => {
				public_list.insert(Api::Traces);
				public_list.insert(Api::ParityPubSub);
				public_list
			},
			ApiSet::IpcContext => {
				public_list.insert(Api::Traces);
				public_list.insert(Api::ParityPubSub);
				public_list.insert(Api::ParityAccounts);
				public_list
			},
			ApiSet::SafeContext => {
				public_list.insert(Api::Traces);
				public_list.insert(Api::ParityPubSub);
				public_list.insert(Api::ParityAccounts);
				public_list.insert(Api::ParitySet);
				public_list.insert(Api::Signer);
				public_list
			},
			ApiSet::All => {
				public_list.insert(Api::Traces);
				public_list.insert(Api::ParityPubSub);
				public_list.insert(Api::ParityAccounts);
				public_list.insert(Api::ParitySet);
				public_list.insert(Api::Signer);
				public_list.insert(Api::Personal);
				public_list
			},
			ApiSet::PubSub => [
				Api::Eth,
				Api::Parity,
				Api::ParityAccounts,
				Api::ParitySet,
				Api::Traces,
			].into_iter().cloned().collect()
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpConfiguration {
	pub enabled: bool,
	pub interface: String,
	pub port: u16,
	pub apis: ApiSet,
	pub cors: Option<Vec<String>>,
	pub hosts: Option<Vec<String>>,
	pub server_threads: Option<usize>,
	pub processing_threads: usize,
}


/// HTTP RPC server impl-independent metadata extractor
pub trait HttpMetaExtractor: Send + Sync + 'static {
	/// Type of Metadata
	type Metadata: jsonrpc_core::Metadata;
	/// Extracts metadata from given params.
	fn read_metadata(&self, origin: Option<String>, user_agent: Option<String>, dapps_origin: Option<String>) -> Self::Metadata;
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
			apis: ApiSet::UnsafeContext,
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

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			// Error::Whisper(ref e) => write!(f, "{}", e),
			Error::Docopt(ref e) => write!(f, "{}", e),
			Error::Io(ref e) => write!(f, "{}", e),
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
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.argv(command).deserialize())?;

	let target_message_pool_size = args.flag_whisper_pool_size * 1024 * 1024;



	let manager = Arc::new(FilterManager::new()?);
	let whisper_handler = Arc::new(WhisperNetwork::new(target_message_pool_size, manager.clone()));



	let mut service = NetworkService::new(NetworkConfiguration::new_local(), None).expect("Error creating network service");
	service.start().expect("Error starting service");
	service.register_protocol(whisper_handler, whisper_net::PROTOCOL_ID, whisper_net::PACKET_COUNT, whisper_net::SUPPORTED_VERSIONS);
	service.register_protocol(Arc::new(whisper_net::ParityExtensions), whisper_net::PARITY_PROTOCOL_ID, whisper_net::PACKET_COUNT, whisper_net::SUPPORTED_VERSIONS);
	// Arc::new(whisper_net::ParityExtensions),
	// ou bien via factory

let deps_for_rpc_apis = Arc::new(rpc_apis::FullDependencies {
		signer_service: signer_service,
		snapshot: snapshot_service.clone(),
		client: client.clone(),
		sync: sync_provider.clone(),
		health: node_health,
		net: manage_network.clone(),
		secret_store: secret_store,
		miner: miner.clone(),
		external_miner: external_miner.clone(),
		logger: logger.clone(),
		settings: Arc::new(cmd.net_settings.clone()),
		net_service: manage_network.clone(),
		updater: updater.clone(),
		geth_compatibility: cmd.geth_compatibility,
		dapps_service: dapps_service,
		dapps_address: cmd.dapps_conf.address(cmd.http_conf.address()),
		ws_address: cmd.ws_conf.address(),
		fetch: fetch.clone(),
		remote: event_loop.remote(),
		whisper_rpc: whisper_factory,
	}); //<-- I don't need this
	let rpc_stats = Arc::new(informant::RpcStats::default());
	let event_loop = EventLoop::spawn();
	let deps = Dependencies {
		apis: deps_for_rpc_apis.clone(),
		remote: event_loop.raw_remote(),
		stats: rpc_stats.clone(),
		pool: None, // @todo
	};

	let http_conf = HttpConfiguration::default();
	let _http_server = new_http("HTTP JSON-RPC", "jsonrpc", http_conf.clone(), &deps, None); // "?;" => todo error handling
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










// attached protos est vide au début
// pub fn setup(target_pool_size: usize, protos: &mut Vec<AttachedProtocol>)
// 	-> io::Result<Option<RpcFactory>>
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





pub fn new_http<D: RpcApisDependencies>(
	id: &str,
	options: &str,
	conf: HttpConfiguration,
	deps: &Dependencies<D>,
	middleware: Option<dapps::Middleware>,
) -> Result<Option<HttpServer>, String> {
	if !conf.enabled {
		return Ok(None);
	}

	let domain = DAPPS_DOMAIN;
	let http_address = (conf.interface, conf.port);
	let url = format!("{}:{}", http_address.0, http_address.1);
	let addr = url.parse().map_err(|_| format!("Invalid {} listen host/port given: {}", id, url))?;
	let pool = deps.pool.clone();
	let handler = setup_apis(conf.apis, deps, pool);
	let remote = deps.remote.clone();

	let cors_domains = into_domains(conf.cors);
	let allowed_hosts = into_domains(with_domain(conf.hosts, domain, &[Some(http_address)]));

	let start_result = parity_rpc::start_http(
		&addr,
		cors_domains,
		allowed_hosts,
		handler,
		remote,
		parity_rpc::RpcExtractor,
		match (conf.server_threads, middleware) {
			(Some(threads), None) => parity_rpc::HttpSettings::Threads(threads),
			(None, middleware) => parity_rpc::HttpSettings::Dapps(middleware),
			(Some(_), Some(_)) => {
				return Err("Dapps and fast multi-threaded RPC server cannot be enabled at the same time.".into())
			},
		}
	);

	match start_result {
		Ok(server) => Ok(Some(server)),
		Err(parity_rpc::HttpServerError::Io(ref err)) if err.kind() == io::ErrorKind::AddrInUse => Err(
			format!("{} address {} is already in use, make sure that another instance of an Ethereum client is not running or change the address using the --{}-port and --{}-interface options.", id, url, options, options)
		),
		Err(e) => Err(format!("{} error: {:?}", id, e)),
	}
}


/// Start http server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_http<M, S, H, T, R>(
	addr: &SocketAddr,
	cors_domains: http::DomainsValidation<http::AccessControlAllowOrigin>,
	allowed_hosts: http::DomainsValidation<http::Host>,
	handler: H,
	remote: tokio_core::reactor::Remote,
	extractor: T,
	settings: HttpSettings<R>,
) -> Result<HttpServer, HttpServerError> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: HttpMetaExtractor<Metadata=M>,
	R: RequestMiddleware,
{
	Ok(match settings {
		HttpSettings::Dapps(middleware) => {
			let mut builder = http::ServerBuilder::new(handler)
				.event_loop_remote(remote)
				.meta_extractor(http_common::HyperMetaExtractor::new(extractor))
				.cors(cors_domains.into())
				.allowed_hosts(allowed_hosts.into());

			if let Some(dapps) = middleware {
				builder = builder.request_middleware(dapps)
			}
			builder.start_http(addr)
				.map(HttpServer::Hyper)?
		},
		HttpSettings::Threads(threads) => {
			minihttp::ServerBuilder::new(handler)
				.threads(threads)
				.meta_extractor(http_common::MiniMetaExtractor::new(extractor))
				.cors(cors_domains.into())
				.allowed_hosts(allowed_hosts.into())
				.start_http(addr)
				.map(HttpServer::Mini)?
		},
	})
}



fn with_domain(items: Option<Vec<String>>, domain: &str, addresses: &[Option<(String, u16)>]) -> Option<Vec<String>> {
	items.map(move |items| {
		let mut items = items.into_iter().collect::<HashSet<_>>();
		for address in addresses {
			if let Some((host, port)) = address.clone() {
				items.insert(format!("{}:{}", host, port));
				items.insert(format!("{}:{}", host.replace("127.0.0.1", "localhost"), port));
				items.insert(format!("http://*.{}:{}", domain, port));
				items.insert(format!("http://*.{}", domain)); //proxypac
			}
		}
		items.into_iter().collect()
	})
}

fn setup_apis<D>(apis: ApiSet, deps: &Dependencies<D>, pool: Option<CpuPool>) -> jsonrpc_core::MetaIoHandler<Metadata, Middleware<D::Notifier>>
	where D: RpcApisDependencies
{
	let mut handler = jsonrpc_core::MetaIoHandler::with_middleware(
		Middleware::new(deps.stats.clone(), deps.apis.activity_notifier(), pool)
	);
	let apis = apis.list_apis();
	deps.apis.extend_with_set(&mut handler, &apis);

	handler
}


fn into_domains<T: From<String>>(items: Option<Vec<String>>) -> DomainsValidation<T> {
	items.map(|vals| vals.into_iter().map(T::from).collect()).into()
}
