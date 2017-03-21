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

//! Ethcore rpc.
#![warn(missing_docs)]
#![cfg_attr(feature="nightly", feature(plugin))]
#![cfg_attr(feature="nightly", plugin(clippy))]

extern crate futures;
extern crate order_stat;
extern crate rustc_serialize;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate transient_hashmap;

extern crate jsonrpc_core;
pub extern crate jsonrpc_http_server as http;
pub extern crate jsonrpc_ipc_server as ipc;

extern crate ethash;
extern crate ethcore;
extern crate ethcore_io as io;
extern crate ethcore_ipc;
extern crate ethcore_light as light;
extern crate ethcrypto as crypto;
extern crate ethkey;
extern crate ethstore;
extern crate ethsync;
extern crate fetch;
extern crate parity_reactor;
extern crate parity_updater as updater;
extern crate rlp;
extern crate stats;

#[macro_use]
extern crate log;
#[macro_use]
extern crate ethcore_util as util;
#[macro_use]
extern crate jsonrpc_macros;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate ethjson;
#[cfg(test)]
extern crate ethcore_devtools as devtools;

pub mod v1;

pub use ipc::{Server as IpcServer, MetaExtractor as IpcMetaExtractor, RequestContext as IpcRequestContext};
pub use http::{HttpMetaExtractor, Server as HttpServer, Error as HttpServerError, AccessControlAllowOrigin, Host};

pub use v1::{SigningQueue, SignerService, ConfirmationsQueue, NetworkSettings, Metadata, Origin, informant, dispatch};
pub use v1::block_import::is_major_importing;

use std::net::SocketAddr;
use http::tokio_core;

/// Start http server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_http<M, S, H, T>(
	addr: &SocketAddr,
	cors_domains: http::DomainsValidation<http::AccessControlAllowOrigin>,
	allowed_hosts: http::DomainsValidation<http::Host>,
	handler: H,
	remote: tokio_core::reactor::Remote,
	extractor: T,
) -> Result<HttpServer, HttpServerError> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: HttpMetaExtractor<M>,
{
	http::ServerBuilder::new(handler)
		.event_loop_remote(remote)
		.meta_extractor(extractor)
		.cors(cors_domains.into())
		.allowed_hosts(allowed_hosts.into())
		.start_http(addr)
}

/// Start ipc server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_ipc<M, S, H, T>(
	addr: &str,
	handler: H,
	remote: tokio_core::reactor::Remote,
	extractor: T,
) -> ::std::io::Result<ipc::Server> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	H: Into<jsonrpc_core::MetaIoHandler<M, S>>,
	T: IpcMetaExtractor<M>,
{
	ipc::ServerBuilder::new(handler)
		.event_loop_remote(remote)
		.session_metadata_extractor(extractor)
		.start(addr)
}
