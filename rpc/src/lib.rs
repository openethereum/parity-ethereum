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

//! Parity Ethereum JSON-RPC Servers (WS, HTTP, IPC).

#![warn(missing_docs, unused_extern_crates)]
#![cfg_attr(feature = "cargo-clippy", warn(clippy::all, clippy::pedantic))]
#![cfg_attr(
	feature = "cargo-clippy",
	allow(
		// things are often more readable this way
		clippy::cast_lossless,
		clippy::module_name_repetitions,
		clippy::single_match_else,
		clippy::type_complexity,
		clippy::use_self,
		// not practical
		clippy::match_bool,
		clippy::needless_pass_by_value,
		clippy::similar_names,
		// don't require markdown syntax for docs
		clippy::doc_markdown,
	),
	warn(clippy::indexing_slicing)
)]

#[macro_use]
extern crate futures;

extern crate ansi_term;
extern crate cid;
extern crate itertools;
extern crate machine;
extern crate multihash;
extern crate order_stat;
extern crate parking_lot;
extern crate rand;
extern crate rustc_hex;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate tokio_timer;
extern crate transient_hashmap;

extern crate jsonrpc_core;
extern crate jsonrpc_derive;
extern crate jsonrpc_http_server as http;
extern crate jsonrpc_ipc_server as ipc;
extern crate jsonrpc_pubsub;

extern crate common_types as types;
extern crate ethash;
extern crate ethcore;
extern crate fastmap;
extern crate parity_bytes as bytes;
extern crate parity_crypto as crypto;
extern crate ethcore_light as light;
extern crate ethcore_logger;
extern crate ethcore_miner as miner;
extern crate ethcore_network as network;
extern crate ethcore_private_tx;
extern crate ethcore_sync as sync;
extern crate ethereum_types;
extern crate ethkey;
extern crate ethstore;
extern crate fetch;
extern crate keccak_hash as hash;
extern crate parity_runtime;
extern crate parity_updater as updater;
extern crate parity_version as version;
extern crate eip_712;
extern crate rlp;
extern crate account_state;
extern crate stats;
extern crate tempdir;
extern crate trace;
extern crate vm;

#[cfg(any(test, feature = "ethcore-accounts"))]
extern crate ethcore_accounts as accounts;

#[cfg(any(test, feature = "ethcore-accounts"))]
extern crate tiny_keccak;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate rand_xorshift;

#[cfg(test)]
extern crate ethjson;
#[cfg(test)]
extern crate transaction_pool as txpool;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
#[macro_use]
extern crate macros;

#[cfg(test)]
extern crate fake_fetch;

#[cfg(test)]
extern crate ethcore_io as io;
#[cfg(test)]
extern crate client_traits;

pub extern crate jsonrpc_ws_server as ws;

mod authcodes;
mod http_common;
pub mod v1;

pub mod tests;

pub use jsonrpc_core::{FutureOutput, FutureResult, FutureResponse, FutureRpcResult};
pub use jsonrpc_pubsub::Session as PubSubSession;
pub use ipc::{Server as IpcServer, MetaExtractor as IpcMetaExtractor, RequestContext as IpcRequestContext};
pub use http::{
	hyper,
	RequestMiddleware, RequestMiddlewareAction,
	AccessControlAllowOrigin, Host, DomainsValidation, cors::AccessControlAllowHeaders
};

pub use v1::{NetworkSettings, Metadata, Origin, informant, dispatch, signer};
pub use v1::block_import::{is_major_importing_or_waiting};
pub use v1::PubSubSyncStatus;
pub use v1::extractors::{RpcExtractor, WsExtractor, WsStats, WsDispatcher};
pub use authcodes::{AuthCodes, TimeProvider};
pub use http_common::HttpMetaExtractor;

use std::net::SocketAddr;

/// RPC HTTP Server instance
pub type HttpServer = http::Server;

/// Start http server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_http<M, S, H, T>(
	addr: &SocketAddr,
	cors_domains: http::DomainsValidation<http::AccessControlAllowOrigin>,
	allowed_hosts: http::DomainsValidation<http::Host>,
	handler: H,
	extractor: T,
	threads: usize,
	max_payload: usize,
	keep_alive: bool,
) -> ::std::io::Result<HttpServer> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: HttpMetaExtractor<Metadata=M>,
{
	let extractor = http_common::MetaExtractor::new(extractor);
	Ok(http::ServerBuilder::with_meta_extractor(handler, extractor)
		.keep_alive(keep_alive)
		.threads(threads)
		.cors(cors_domains)
		.allowed_hosts(allowed_hosts)
		.health_api(("/api/health", "parity_nodeStatus"))
		.cors_allow_headers(AccessControlAllowHeaders::Any)
		.max_request_body_size(max_payload * 1024 * 1024)
		.start_http(addr)?)
}

/// Same as `start_http`, but takes an additional `middleware` parameter that is introduced as a
/// hyper middleware.
pub fn start_http_with_middleware<M, S, H, T, R>(
	addr: &SocketAddr,
	cors_domains: http::DomainsValidation<http::AccessControlAllowOrigin>,
	allowed_hosts: http::DomainsValidation<http::Host>,
	handler: H,
	extractor: T,
	middleware: R,
	threads: usize,
	max_payload: usize,
	keep_alive: bool,
) -> ::std::io::Result<HttpServer> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: HttpMetaExtractor<Metadata=M>,
	R: RequestMiddleware,
{
	let extractor = http_common::MetaExtractor::new(extractor);
	Ok(http::ServerBuilder::with_meta_extractor(handler, extractor)
		.keep_alive(keep_alive)
		.threads(threads)
		.cors(cors_domains)
		.allowed_hosts(allowed_hosts)
		.cors_allow_headers(AccessControlAllowHeaders::Any)
		.max_request_body_size(max_payload * 1024 * 1024)
		.request_middleware(middleware)
		.start_http(addr)?)
}

/// Start ipc server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_ipc<M, S, H, T>(
	addr: &str,
	handler: H,
	extractor: T,
) -> ::std::io::Result<ipc::Server> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: IpcMetaExtractor<M>,
{
	ipc::ServerBuilder::with_meta_extractor(handler, extractor)
		.start(addr)
}

/// Start WS server and return `Server` handle.
pub fn start_ws<M, S, H, T, U, V>(
	addr: &SocketAddr,
	handler: H,
	allowed_origins: ws::DomainsValidation<ws::Origin>,
	allowed_hosts: ws::DomainsValidation<ws::Host>,
	max_connections: usize,
	extractor: T,
	middleware: V,
	stats: U,
) -> Result<ws::Server, ws::Error> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: ws::MetaExtractor<M>,
	U: ws::SessionStats,
	V: ws::RequestMiddleware,
{
	ws::ServerBuilder::with_meta_extractor(handler, extractor)
		.request_middleware(middleware)
		.allowed_origins(allowed_origins)
		.allowed_hosts(allowed_hosts)
		.max_connections(max_connections)
		.session_stats(stats)
		.start(addr)
}
