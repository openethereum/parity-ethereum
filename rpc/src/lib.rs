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
extern crate ethcore_util as util;
extern crate ethcore;
extern crate ethsync;
extern crate ethminer;
extern crate transient_hashmap;

use std::sync::Arc;
use std::net::SocketAddr;
use self::jsonrpc_core::{IoHandler, IoDelegate};

pub use jsonrpc_http_server::{Server, RpcServerError};
pub mod v1;

/// Http server.
pub struct RpcServer {
	handler: Arc<IoHandler>,
}

impl RpcServer {
	/// Construct new http server object.
	pub fn new() -> RpcServer {
		RpcServer {
			handler: Arc::new(IoHandler::new()),
		}
	}

	/// Add io delegate.
	pub fn add_delegate<D>(&self, delegate: IoDelegate<D>) where D: Send + Sync + 'static {
		self.handler.add_delegate(delegate);
	}

	/// Start http server asynchronously and returns result with `Server` handle on success or an error.
	pub fn start_http(&self, addr: &SocketAddr, cors_domain: Option<String>) -> Result<Server, RpcServerError> {
		let cors_domain = cors_domain.to_owned();
		Server::start(addr, self.handler.clone(), cors_domain.map(jsonrpc_http_server::AccessControlAllowOrigin::Value))
	}
}
