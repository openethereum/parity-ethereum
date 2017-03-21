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

extern crate semver;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;

extern crate ethcore_io as io;
extern crate ethcore;
extern crate ethkey;
extern crate ethcrypto as crypto;
extern crate ethstore;
extern crate ethsync;
extern crate ethash;
extern crate ethcore_light as light;
extern crate transient_hashmap;
extern crate jsonrpc_ipc_server as ipc;
extern crate ethcore_ipc;
extern crate time;
extern crate rlp;
extern crate fetch;
extern crate futures;
extern crate order_stat;
extern crate parity_updater as updater;
extern crate parity_reactor;
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

use std::sync::Arc;
use std::net::SocketAddr;
use io::PanicHandler;
use jsonrpc_core::reactor::RpcHandler;

pub use ipc::{Server as IpcServer, Error as IpcServerError};
pub use jsonrpc_http_server::{ServerBuilder, Server, RpcServerError, HttpMetaExtractor};
pub mod v1;
pub use v1::{SigningQueue, SignerService, ConfirmationsQueue, NetworkSettings, Metadata, Origin, informant, dispatch};
pub use v1::block_import::is_major_importing;

/// Start http server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_http<M, T, S>(
	addr: &SocketAddr,
	cors_domains: Option<Vec<String>>,
	allowed_hosts: Option<Vec<String>>,
	panic_handler: Arc<PanicHandler>,
	handler: RpcHandler<M, S>,
	extractor: T,
) -> Result<Server, RpcServerError> where
	M: jsonrpc_core::Metadata,
	S: jsonrpc_core::Middleware<M>,
	T: HttpMetaExtractor<M>,
{

	let cors_domains = cors_domains.map(|domains| {
		domains.into_iter()
			.map(|v| match v.as_str() {
				"*" => jsonrpc_http_server::AccessControlAllowOrigin::Any,
				"null" => jsonrpc_http_server::AccessControlAllowOrigin::Null,
				v => jsonrpc_http_server::AccessControlAllowOrigin::Value(v.into()),
			})
			.collect()
	});

	ServerBuilder::with_rpc_handler(handler)
		.meta_extractor(Arc::new(extractor))
		.cors(cors_domains.into())
		.allowed_hosts(allowed_hosts.into())
		.panic_handler(move || {
			panic_handler.notify_all("Panic in RPC thread.".to_owned());
		})
		.start_http(addr)
}

/// Start ipc server asynchronously and returns result with `Server` handle on success or an error.
pub fn start_ipc<M: jsonrpc_core::Metadata, S: jsonrpc_core::Middleware<M>>(
	addr: &str,
	handler: RpcHandler<M, S>,
) -> Result<ipc::Server<M, S>, ipc::Error> {
	let server = ipc::Server::with_rpc_handler(addr, handler)?;
	server.run_async()?;
	Ok(server)
}
