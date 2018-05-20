// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

extern crate multihash;
extern crate cid;
extern crate unicase;

extern crate rlp;
extern crate ethcore;
extern crate ethcore_bytes as bytes;
extern crate ethereum_types;
extern crate jsonrpc_core as core;
extern crate jsonrpc_http_server as http;

pub mod error;
mod route;

use std::thread;
use std::sync::{mpsc, Arc};
use std::net::{SocketAddr, IpAddr};

use core::futures::future::{self, FutureResult};
use core::futures::{self, Future};
use ethcore::client::BlockChainClient;
use http::hyper::header::{self, Vary, ContentType};
use http::hyper::{Method, StatusCode};
use http::hyper::{self, server};
use unicase::Ascii;

use error::ServerError;
use route::Out;

pub use http::{AccessControlAllowOrigin, Host, DomainsValidation};

/// Request/response handler
pub struct IpfsHandler {
	/// Allowed CORS domains
	cors_domains: Option<Vec<AccessControlAllowOrigin>>,
	/// Hostnames allowed in the `Host` request header
	allowed_hosts: Option<Vec<Host>>,
	/// Reference to the Blockchain Client
	client: Arc<BlockChainClient>,
}

impl IpfsHandler {
	pub fn client(&self) -> &BlockChainClient {
		&*self.client
	}

	pub fn new(cors: DomainsValidation<AccessControlAllowOrigin>, hosts: DomainsValidation<Host>, client: Arc<BlockChainClient>) -> Self {
		IpfsHandler {
			cors_domains: cors.into(),
			allowed_hosts: hosts.into(),
			client: client,
		}
	}
	pub fn on_request(&self, req: hyper::Request) -> (Option<header::AccessControlAllowOrigin>, Out) {
		match *req.method() {
			Method::Get | Method::Post => {},
			_ => return (None, Out::Bad("Invalid Request")),
		}

		if !http::is_host_allowed(&req, &self.allowed_hosts) {
			return (None, Out::Bad("Disallowed Host header"));
		}

		let cors_header = http::cors_header(&req, &self.cors_domains);
		if cors_header == http::CorsHeader::Invalid {
			return (None, Out::Bad("Disallowed Origin header"));
		}

		let path = req.uri().path();
		let query = req.uri().query();
		return (cors_header.into(), self.route(path, query));
	}
}

impl server::Service for IpfsHandler {
	type Request = hyper::Request;
	type Response = hyper::Response;
	type Error = hyper::Error;
	type Future = FutureResult<hyper::Response, hyper::Error>;

	fn call(&self, request: Self::Request) -> Self::Future {
		let (cors_header, out) = self.on_request(request);

		let mut res = match out {
			Out::OctetStream(bytes) => {
				hyper::Response::new()
					.with_status(StatusCode::Ok)
					.with_header(ContentType::octet_stream())
					.with_body(bytes)
			},
			Out::NotFound(reason) => {
				hyper::Response::new()
					.with_status(StatusCode::NotFound)
					.with_header(ContentType::plaintext())
					.with_body(reason)
			},
			Out::Bad(reason) => {
				hyper::Response::new()
					.with_status(StatusCode::BadRequest)
					.with_header(ContentType::plaintext())
					.with_body(reason)
			}
		};

		if let Some(cors_header) = cors_header {
			res.headers_mut().set(cors_header);
			res.headers_mut().set(Vary::Items(vec![Ascii::new("Origin".into())]));
		}

		future::ok(res)
	}
}

/// Add current interface (default: "127.0.0.1:5001") to list of allowed hosts
fn include_current_interface(mut hosts: Vec<Host>, interface: String, port: u16) -> Vec<Host> {
	hosts.push(match port {
		80 => interface,
		_ => format!("{}:{}", interface, port),
	}.into());

	hosts
}

#[derive(Debug)]
pub struct Listening {
	close: Option<futures::sync::oneshot::Sender<()>>,
	thread: Option<thread::JoinHandle<()>>,
}

impl Drop for Listening {
	fn drop(&mut self) {
		self.close.take().unwrap().send(()).unwrap();
		let _ = self.thread.take().unwrap().join();
	}
}

pub fn start_server(
	port: u16,
	interface: String,
	cors: DomainsValidation<AccessControlAllowOrigin>,
	hosts: DomainsValidation<Host>,
	client: Arc<BlockChainClient>
) -> Result<Listening, ServerError> {

	let ip: IpAddr = interface.parse().map_err(|_| ServerError::InvalidInterface)?;
	let addr = SocketAddr::new(ip, port);
	let hosts: Option<Vec<_>> = hosts.into();
	let hosts: DomainsValidation<_> = hosts.map(move |hosts| include_current_interface(hosts, interface, port)).into();

	let (close, shutdown_signal) = futures::sync::oneshot::channel::<()>();
	let (tx, rx) = mpsc::sync_channel(1);
	let thread = thread::spawn(move || {
		let send = |res| tx.send(res).expect("rx end is never dropped; qed");
		let server = match server::Http::new().bind(&addr, move || {
			Ok(IpfsHandler::new(cors.clone(), hosts.clone(), client.clone()))
		}) {
			Ok(server) => {
				send(Ok(()));
				server
			},
			Err(err) => {
				send(Err(err));
				return;
			}
		};

		let _ = server.run_until(shutdown_signal.map_err(|_| {}));
	});

	// Wait for server to start successfuly.
	rx.recv().expect("tx end is never dropped; qed")?;

	Ok(Listening {
		close: close.into(),
		thread: thread.into(),
	})
}
