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

#[macro_use]
extern crate mime;
extern crate hyper;
extern crate multihash;
extern crate cid;

extern crate rlp;
extern crate ethcore;
extern crate ethcore_util as util;
extern crate jsonrpc_http_server;

pub mod error;
mod route;

use std::io::Write;
use std::sync::Arc;
use std::net::{SocketAddr, IpAddr};
use error::ServerError;
use route::Out;
use jsonrpc_http_server::cors;
use hyper::server::{Listening, Handler, Request, Response};
use hyper::net::HttpStream;
use hyper::header::{Vary, ContentLength, ContentType, AccessControlAllowOrigin};
use hyper::{Next, Encoder, Decoder, Method, RequestUri, StatusCode};
use ethcore::client::BlockChainClient;


/// Request/response handler
pub struct IpfsHandler {
	/// Response to send out
	out: Out,
	/// How many bytes from the response have been written
	out_progress: usize,
	/// Origin request header
	origin: Option<String>,
	/// Allowed CORS domains
	cors_domains: Option<Vec<AccessControlAllowOrigin>>,
	/// Hostnames allowed in the `Host` request header
	allowed_hosts: Option<Vec<String>>,
	/// Reference to the Blockchain Client
	client: Arc<BlockChainClient>,
}

impl IpfsHandler {
	pub fn client(&self) -> &BlockChainClient {
		&*self.client
	}

	pub fn new(cors: Option<Vec<String>>, hosts: Option<Vec<String>>, client: Arc<BlockChainClient>) -> Self {
		fn origin_to_header(origin: String) -> AccessControlAllowOrigin {
			match origin.as_str() {
				"*" => AccessControlAllowOrigin::Any,
				"null" | "" => AccessControlAllowOrigin::Null,
				_ => AccessControlAllowOrigin::Value(origin),
			}
		}

		IpfsHandler {
			out: Out::Bad("Invalid Request"),
			out_progress: 0,
			origin: None,
			cors_domains: cors.map(|vec| vec.into_iter().map(origin_to_header).collect()),
			allowed_hosts: hosts,
			client: client,
		}
	}

	fn is_host_allowed(&self, req: &Request<HttpStream>) -> bool {
		match self.allowed_hosts {
			Some(ref hosts) => jsonrpc_http_server::is_host_header_valid(&req, hosts),
			None => true,
		}
	}

	fn is_origin_allowed(&self) -> bool {
		// Check origin header first, no header passed is good news
		let origin = match self.origin {
			Some(ref origin) => origin,
			None => return true,
		};

		let cors_domains = match self.cors_domains {
			Some(ref domains) => domains,
			None => return false,
		};

		cors_domains.iter().any(|domain| match *domain {
			AccessControlAllowOrigin::Value(ref allowed) => origin == allowed,
			AccessControlAllowOrigin::Any => true,
			AccessControlAllowOrigin::Null => origin == "",
		})
	}
}

/// Implement Hyper's HTTP handler
impl Handler<HttpStream> for IpfsHandler {
	fn on_request(&mut self, req: Request<HttpStream>) -> Next {
		if *req.method() != Method::Get {
			return Next::write();
		}

		self.origin = cors::read_origin(&req);

		if !self.is_host_allowed(&req) {
			self.out = Out::Bad("Disallowed Host header");

			return Next::write();
		}

		if !self.is_origin_allowed() {
			self.out = Out::Bad("Disallowed Origin header");

			return Next::write();
		}

		let (path, query) = match *req.uri() {
			RequestUri::AbsolutePath { ref path, ref query } => (path, query.as_ref().map(AsRef::as_ref)),
			_ => return Next::write(),
		};

		self.out = self.route(path, query);

		Next::write()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		Next::write()
	}

	fn on_response(&mut self, res: &mut Response) -> Next {
		use Out::*;

		match self.out {
			OctetStream(ref bytes) => {
				use mime::{Mime, TopLevel, SubLevel};

				// `OctetStream` is not a valid variant, so need to construct
				// the type manually.
				let content_type = Mime(
					TopLevel::Application,
					SubLevel::Ext("octet-stream".into()),
					vec![]
				);

				res.headers_mut().set(ContentLength(bytes.len() as u64));
				res.headers_mut().set(ContentType(content_type));

			},
			NotFound(reason) => {
				res.set_status(StatusCode::NotFound);

				res.headers_mut().set(ContentLength(reason.len() as u64));
				res.headers_mut().set(ContentType(mime!(Text/Plain)));
			},
			Bad(reason) => {
				res.set_status(StatusCode::BadRequest);

				res.headers_mut().set(ContentLength(reason.len() as u64));
				res.headers_mut().set(ContentType(mime!(Text/Plain)));
			}
		}

		if let Some(cors_header) = cors::get_cors_header(&self.cors_domains, &self.origin) {
			res.headers_mut().set(cors_header);
			res.headers_mut().set(Vary::Items(vec!["Origin".into()]));
		}

		Next::write()
	}

	fn on_response_writable(&mut self, transport: &mut Encoder<HttpStream>) -> Next {
		use Out::*;

		// Get the data to write as a byte slice
		let data = match self.out {
			OctetStream(ref bytes) => &bytes,
			NotFound(reason) | Bad(reason) => reason.as_bytes(),
		};

		write_chunk(transport, &mut self.out_progress, data)
	}
}

/// Attempt to write entire `data` from current `progress`
fn write_chunk<W: Write>(transport: &mut W, progress: &mut usize, data: &[u8]) -> Next {
	// Skip any bytes that have already been written
	let chunk = &data[*progress..];

	// Write an get the amount of bytes written. End the connection in case of an error.
	let written = match transport.write(chunk) {
		Ok(written) => written,
		Err(_) => return Next::end(),
	};

	*progress += written;

	// Close the connection if the entire remaining chunk has been written
	if written < chunk.len() {
		Next::write()
	} else {
		Next::end()
	}
}

/// Add current interface (default: "127.0.0.1:5001") to list of allowed hosts
fn include_current_interface(mut hosts: Vec<String>, interface: String, port: u16) -> Vec<String> {
	hosts.push(match port {
		80 => interface,
		_ => format!("{}:{}", interface, port),
	});

	hosts
}

pub fn start_server(
	port: u16,
	interface: String,
	cors: Option<Vec<String>>,
	hosts: Option<Vec<String>>,
	client: Arc<BlockChainClient>
) -> Result<Listening, ServerError> {

	let ip: IpAddr = interface.parse().map_err(|_| ServerError::InvalidInterface)?;
	let addr = SocketAddr::new(ip, port);
	let hosts = hosts.map(move |hosts| include_current_interface(hosts, interface, port));

	Ok(
		hyper::Server::http(&addr)?
			.handle(move |_| IpfsHandler::new(cors.clone(), hosts.clone(), client.clone()))
			.map(|(listening, srv)| {

				::std::thread::spawn(move || {
					srv.run();
				});

				listening
			})?
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn write_chunk_to_vec() {
		let mut transport = Vec::new();
		let mut progress = 0;

		let _ = write_chunk(&mut transport, &mut progress, b"foobar");

		assert_eq!(b"foobar".to_vec(), transport);
		assert_eq!(6, progress);
	}

	#[test]
	fn write_chunk_to_vec_part() {
		let mut transport = Vec::new();
		let mut progress = 3;

		let _ = write_chunk(&mut transport, &mut progress, b"foobar");

		assert_eq!(b"bar".to_vec(), transport);
		assert_eq!(6, progress);
	}

	#[test]
	fn write_chunk_to_array() {
		use std::io::Cursor;

		let mut buf = [0u8; 3];
		let mut progress = 0;

		{
			let mut transport: Cursor<&mut [u8]> = Cursor::new(&mut buf);
			let _ = write_chunk(&mut transport, &mut progress, b"foobar");
		}

		assert_eq!(*b"foo", buf);
		assert_eq!(3, progress);

		{
			let mut transport: Cursor<&mut [u8]> = Cursor::new(&mut buf);
			let _ = write_chunk(&mut transport, &mut progress, b"foobar");
		}

		assert_eq!(*b"bar", buf);
		assert_eq!(6, progress);
	}
}
