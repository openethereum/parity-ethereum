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

mod error;
mod handler;

use std::io::Write;
use std::sync::Arc;
use error::ServerError;
use handler::{IpfsHandler, Out};
use hyper::server::{Listening, Handler, Request, Response};
use hyper::net::HttpStream;
use hyper::header::{ContentLength, ContentType};
use hyper::{Next, Encoder, Decoder, Method, RequestUri, StatusCode};
use ethcore::client::BlockChainClient;

/// Implement Hyper's HTTP handler
impl Handler<HttpStream> for IpfsHandler {
	fn on_request(&mut self, req: Request<HttpStream>) -> Next {
		if *req.method() != Method::Get {
			return Next::write();
		}

		let (path, query) = match *req.uri() {
			RequestUri::AbsolutePath { ref path, ref query } => (path, query.as_ref().map(AsRef::as_ref)),
			_ => return Next::write(),
		};

		self.route(path, query)
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

				Next::write()
			},
			NotFound(reason) => {
				res.set_status(StatusCode::NotFound);

				res.headers_mut().set(ContentLength(reason.len() as u64));
				res.headers_mut().set(ContentType(mime!(Text/Plain)));

				Next::write()
			},
			Bad(reason) => {
				res.set_status(StatusCode::BadRequest);

				res.headers_mut().set(ContentLength(reason.len() as u64));
				res.headers_mut().set(ContentType(mime!(Text/Plain)));

				Next::write()
			}
		}
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

fn write_chunk<W: Write>(transport: &mut W, progress: &mut usize, data: &[u8]) -> Next {
	// Skip any bytes that have already been written
	let chunk = &data[*progress..];

	// Write an get written count
	let written = match transport.write(chunk) {
		Ok(written) => written,
		Err(_) => return Next::end(),
	};

	*progress += written;

	// Close the connection if the entire chunk has been written, otherwise increment progress
	if written < chunk.len() {
		Next::write()
	} else {
		Next::end()
	}
}

pub fn start_server(client: Arc<BlockChainClient>) -> Result<Listening, ServerError> {
	let addr = "0.0.0.0:5001".parse().expect("can't fail on static input; qed");

	Ok(
		hyper::Server::http(&addr)?
			.handle(move |_| IpfsHandler::new(client.clone()))
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
