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

//! `WebSockets` server.

use ws;
use std;
use std::thread;
use std::default::Default;
use std::ops::Drop;
use std::sync::Arc;
use std::net::SocketAddr;
use util::panics::{PanicHandler, OnPanicListener, MayPanic};
use jsonrpc_core::{IoHandler, IoDelegate};
use rpc::{Extendable, ConfirmationsQueue};

mod session;

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

/// Builder for `WebSockets` server
pub struct ServerBuilder {
	queue: Arc<ConfirmationsQueue>,
	handler: Arc<IoHandler>,
}

impl Extendable for ServerBuilder {
	fn add_delegate<D: Send + Sync + 'static>(&self, delegate: IoDelegate<D>) {
		self.handler.add_delegate(delegate);
	}
}

impl ServerBuilder {
	/// Creates new `ServerBuilder`
	pub fn new(queue: Arc<ConfirmationsQueue>) -> Self {
		ServerBuilder {
			queue: queue,
			handler: Arc::new(IoHandler::new()),
		}
	}

	/// Starts a new `WebSocket` server in separate thread.
	/// Returns a `Server` handle which closes the server when droped.
	pub fn start(self, addr: SocketAddr) -> Result<Server, ServerError> {
		Server::start(addr, self.handler).and_then(|(server, broadcaster)| {
			// Fire up queue notifications broadcasting
			let queue = self.queue.clone();
			thread::spawn(move || {
				queue.start_listening(|_message| {
					broadcaster.send("new_message").unwrap();
				}).expect("It's the only place we are running start_listening. It shouldn't fail.");
			}).expect("We should be able to create the thread");

			Ok(server)
		})
	}
}

/// `WebSockets` server implementation.
pub struct Server {
	handle: Option<thread::JoinHandle<ws::WebSocket<session::Factory>>>,
	panic_handler: Arc<PanicHandler>,
}

impl Server {
	/// Starts a new `WebSocket` server in separate thread.
	/// Returns a `Server` handle which closes the server when droped.
	fn start(addr: SocketAddr, handler: Arc<IoHandler>) -> Result<(Server, ws::Sender), ServerError> {
		let config = {
			let mut config = ws::Settings::default();
			config.max_connections = 5;
			config.method_strict = true;
			config
		};

		// Create WebSocket
		let ws = try!(ws::Builder::new().with_settings(config).build(session::Factory::new(handler)));

		let panic_handler = PanicHandler::new_in_arc();
		let ph = panic_handler.clone();
		let broadcaster = ws.broadcaster();
		// Spawn a thread with event loop
		let handle = thread::spawn(move || {
			ph.catch_panic(move || {
				ws.listen(addr).unwrap()
			}).unwrap()
		});

		// Return a handle
		Ok((Server {
			handle: Some(handle),
			panic_handler: panic_handler,
		}, broadcaster))
	}
}

impl MayPanic for Server {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}

impl Drop for Server {
	fn drop(&mut self) {
		self.handle.take().unwrap().join().unwrap();
	}
}
