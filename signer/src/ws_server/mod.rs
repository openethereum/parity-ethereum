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

//! `WebSockets` server.

use ws;
use std::default::Default;
use std::net::SocketAddr;
use std::ops::Drop;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std;

use io::{PanicHandler, OnPanicListener, MayPanic};
use jsonrpc_core::{Metadata, Middleware};
use jsonrpc_core::reactor::RpcHandler;
use rpc::{ConfirmationsQueue};
use rpc::informant::RpcStats;

mod session;

pub use self::session::MetaExtractor;

/// Signer startup error
#[derive(Debug)]
pub enum ServerError {
	/// Wrapped `std::io::Error`
	IoError(std::io::Error),
	/// Other `ws-rs` error
	WebSocket(ws::Error)
}

impl From<ws::Error> for ServerError {
	fn from(err: ws::Error) -> Self {
		match err.kind {
			ws::ErrorKind::Io(e) => ServerError::IoError(e),
			_ => ServerError::WebSocket(err),
		}
	}
}

/// Dummy metadata extractor
#[derive(Clone)]
pub struct NoopExtractor;
impl<M: Metadata> session::MetaExtractor<M> for NoopExtractor {}

/// Builder for `WebSockets` server
pub struct ServerBuilder {
	queue: Arc<ConfirmationsQueue>,
	authcodes_path: PathBuf,
	skip_origin_validation: bool,
	stats: Option<Arc<RpcStats>>,
}

impl ServerBuilder {
	/// Creates new `ServerBuilder`
	pub fn new(queue: Arc<ConfirmationsQueue>, authcodes_path: PathBuf) -> Self {
		ServerBuilder {
			queue: queue,
			authcodes_path: authcodes_path,
			skip_origin_validation: false,
			stats: None,
		}
	}

	/// If set to `true` server will not verify Origin of incoming requests.
	/// Not recommended. Use only for development.
	pub fn skip_origin_validation(mut self, skip: bool) -> Self {
		self.skip_origin_validation = skip;
		self
	}

	/// Configure statistic collection
	pub fn stats(mut self, stats: Arc<RpcStats>) -> Self {
		self.stats = Some(stats);
		self
	}

	/// Starts a new `WebSocket` server in separate thread.
	/// Returns a `Server` handle which closes the server when droped.
	pub fn start<M: Metadata, S: Middleware<M>>(self, addr: SocketAddr, handler: RpcHandler<M, S>) -> Result<Server, ServerError> {
		self.start_with_extractor(addr, handler, NoopExtractor)
	}

	/// Starts a new `WebSocket` server in separate thread.
	/// Returns a `Server` handle which closes the server when droped.
	pub fn start_with_extractor<M: Metadata, S: Middleware<M>, T: session::MetaExtractor<M>>(
		self,
		addr: SocketAddr,
		handler: RpcHandler<M, S>,
		meta_extractor: T,
	) -> Result<Server, ServerError> {
		Server::start(
			addr,
			handler,
			self.queue,
			self.authcodes_path,
			self.skip_origin_validation,
			self.stats,
			meta_extractor,
		)
	}

}

/// `WebSockets` server implementation.
pub struct Server {
	handle: Option<thread::JoinHandle<()>>,
	broadcaster_handle: Option<thread::JoinHandle<()>>,
	queue: Arc<ConfirmationsQueue>,
	panic_handler: Arc<PanicHandler>,
	addr: SocketAddr,
}

impl Server {
	/// Returns the address this server is listening on
	pub fn addr(&self) -> &SocketAddr {
		&self.addr
	}

	/// Starts a new `WebSocket` server in separate thread.
	/// Returns a `Server` handle which closes the server when droped.
	fn start<M: Metadata, S: Middleware<M>, T: session::MetaExtractor<M>>(
		addr: SocketAddr,
		handler: RpcHandler<M, S>,
		queue: Arc<ConfirmationsQueue>,
		authcodes_path: PathBuf,
		skip_origin_validation: bool,
		stats: Option<Arc<RpcStats>>,
		meta_extractor: T,
	) -> Result<Server, ServerError> {
		let config = {
			let mut config = ws::Settings::default();
			// accept only handshakes beginning with GET
			config.method_strict = true;
			// Was shutting down server when suspending on linux:
			config.shutdown_on_interrupt = false;
			config
		};

		// Create WebSocket
		let origin = format!("{}", addr);
		let port = addr.port();
		let ws = ws::Builder::new().with_settings(config).build(
			session::Factory::new(handler, origin, port, authcodes_path, skip_origin_validation, stats, meta_extractor)
		)?;

		let panic_handler = PanicHandler::new_in_arc();
		let ph = panic_handler.clone();
		let broadcaster = ws.broadcaster();

		// Spawn a thread with event loop
		let handle = thread::spawn(move || {
			ph.catch_panic(move || {
				match ws.listen(addr).map_err(ServerError::from) {
					Err(ServerError::IoError(io)) => die(format!(
						"Signer: Could not start listening on specified address. Make sure that no other instance is running on Signer's port. Details: {:?}",
						io
					)),
					Err(any_error) => die(format!(
						"Signer: Unknown error occurred when starting Signer. Details: {:?}",
						any_error
					)),
					Ok(server) => server,
				}
			}).unwrap();
		});

		// Spawn a thread for broadcasting
		let ph = panic_handler.clone();
		let q = queue.clone();
		let broadcaster_handle = thread::spawn(move || {
			ph.catch_panic(move || {
				q.start_listening(|_message| {
					// TODO [ToDr] Some better structure here for messages.
					broadcaster.send("new_message").unwrap();
				}).expect("It's the only place we are running start_listening. It shouldn't fail.");
				let res = broadcaster.shutdown();

				if let Err(e) = res {
					warn!("Signer: Broadcaster was not closed cleanly. Details: {:?}", e);
				}
			}).unwrap()
		});

		// Return a handle
		Ok(Server {
			handle: Some(handle),
			broadcaster_handle: Some(broadcaster_handle),
			queue: queue,
			panic_handler: panic_handler,
			addr: addr,
		})
	}
}

impl MayPanic for Server {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}

impl Drop for Server {
	fn drop(&mut self) {
		self.queue.finish();
		self.broadcaster_handle.take().unwrap().join().unwrap();
		self.handle.take().unwrap().join().unwrap();
	}
}

fn die(msg: String) -> ! {
	println!("ERROR: {}", msg);
	std::process::exit(1);
}
