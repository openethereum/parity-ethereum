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

#![warn(missing_docs)]
#![cfg_attr(all(nightly, feature="dev"), feature(plugin))]
#![cfg_attr(all(nightly, feature="dev"), plugin(clippy))]

//! Signer module
//!
//! This module manages your private keys and accounts/identities
//! that can be used within Dapps.
//!
//! It exposes API (over `WebSockets`) accessed by Signer UIs.
//! Each transaction sent by Dapp is broadcasted to Signer UIs
//! and their responsibility is to confirm (or confirm and sign)
//! the transaction for you.
//!
//! ```
//! extern crate jsonrpc_core;
//! extern crate ethcore_signer;
//! extern crate ethcore_rpc;
//!
//! use std::sync::Arc;
//! use jsonrpc_core::IoHandler;
//! use jsonrpc_core::reactor::RpcEventLoop;
//! use ethcore_signer::ServerBuilder;
//! use ethcore_rpc::ConfirmationsQueue;
//!
//!	fn main() {
//!	 let queue = Arc::new(ConfirmationsQueue::default());
//!	 let io = Arc::new(IoHandler::new().into());
//!	 let event_loop = RpcEventLoop::spawn();
//!	 let _server = ServerBuilder::new(queue, "/tmp/authcodes".into())
//!		.start("127.0.0.1:8084".parse().unwrap(), event_loop.handler(io));
//!	}
//! ```

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;

extern crate ethcore_util as util;
extern crate ethcore_rpc as rpc;
extern crate ethcore_io as io;
extern crate jsonrpc_core;
extern crate ws;

extern crate ethcore_devtools as devtools;

mod authcode_store;
mod ws_server;

/// Exported tests for use in signer RPC client testing
pub mod tests;
pub use authcode_store::*;
pub use ws_server::*;
