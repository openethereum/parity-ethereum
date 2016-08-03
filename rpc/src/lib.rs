// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
#![cfg_attr(feature="nightly", feature(custom_derive, custom_attribute, plugin))]
#![cfg_attr(feature="nightly", plugin(serde_macros, clippy))]

#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
#[macro_use]
extern crate ethcore_util as util;
extern crate ethcore;
extern crate ethsync;
extern crate transient_hashmap;
extern crate json_ipc_server as ipc;
extern crate ethcore_ipc;

#[cfg(test)]
extern crate ethjson;
#[cfg(test)]
extern crate ethcore_devtools as devtools;

use std::sync::Arc;
use std::net::SocketAddr;
use util::panics::PanicHandler;
use self::jsonrpc_core::{IoHandler, IoDelegate};

pub use jsonrpc_http_server::{ServerBuilder, Server, RpcServerError};
pub mod v1;
pub use v1::{SigningQueue, ConfirmationsQueue, NetworkSettings};

/// An object that can be extended with `IoDelegates`
pub trait Extendable {
	/// Add `Delegate` to this object.
	fn add_delegate<D: Send + Sync + 'static>(&self, delegate: IoDelegate<D>);
}

/// Http server.
pub struct RpcServer {
	handler: Arc<jsonrpc_core::io::IoHandler>,
}

impl Extendable for RpcServer {
	/// Add io delegate.
	fn add_delegate<D: Send + Sync + 'static>(&self, delegate: IoDelegate<D>) {
		self.handler.add_delegate(delegate);
	}
}

impl RpcServer {
	/// Construct new http server object.
	pub fn new() -> RpcServer {
		RpcServer {
			handler: Arc::new(IoHandler::new()),
		}
	}

	/// Start http server asynchronously and returns result with `Server` handle on success or an error.
	pub fn start_http(
		&self,
		addr: &SocketAddr,
		cors_domains: Option<Vec<String>>,
		allowed_hosts: Option<Vec<String>>,
		panic_handler: Arc<PanicHandler>,
		) -> Result<Server, RpcServerError> {

		let cors_domains = cors_domains.map(|domains| {
			domains.into_iter()
				.map(|v| match v.as_str() {
					"*" => jsonrpc_http_server::AccessControlAllowOrigin::Any,
					"null" => jsonrpc_http_server::AccessControlAllowOrigin::Null,
					v => jsonrpc_http_server::AccessControlAllowOrigin::Value(v.into()),
				})
				.collect()
		});

		ServerBuilder::new(self.handler.clone())
			.cors(cors_domains.into())
			.allowed_hosts(allowed_hosts.into())
			.panic_handler(move || {
				panic_handler.notify_all("Panic in RPC thread.".to_owned());
			})
			.start_http(addr)
	}

	/// Start ipc server asynchronously and returns result with `Server` handle on success or an error.
	pub fn start_ipc(&self, addr: &str) -> Result<ipc::Server, ipc::Error> {
		let server = try!(ipc::Server::new(addr, &self.handler));
		try!(server.run_async());
		Ok(server)
	}
}
