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

//! Content Stream Response

use std::io::{self, Read};

use futures::{self, sink, Sink, Future};
use futures::sync::mpsc;
use hyper::{self, header, mime, StatusCode};

use handlers::add_security_headers;
use Embeddable;

pub struct StreamingHandler<R> {
	initial: Vec<u8>,
	content: R,
	status: StatusCode,
	mimetype: mime::Mime,
	safe_to_embed_on: Embeddable,
}

impl<R: io::Read> StreamingHandler<R> {
	pub fn new(content: R, status: StatusCode, mimetype: mime::Mime, safe_to_embed_on: Embeddable) -> Self {
		StreamingHandler {
			initial: Vec::new(),
			content,
			status,
			mimetype,
			safe_to_embed_on,
		}
	}

	pub fn set_initial_content(&mut self, content: &str) {
		self.initial = content.as_bytes().to_vec();
	}

	pub fn into_response(self) -> (Reading<R>, hyper::Response) {
		let (tx, rx) = hyper::Body::pair();
		let reader = Reading {
			buffer: [0; MAX_CHUNK_SIZE],
			content: io::BufReader::new(self.content),
			sending: tx.send(Ok(self.initial.into())),
		};

		let mut res = hyper::Response::new()
			.with_status(self.status)
			.with_header(header::ContentType(self.mimetype))
			.with_body(rx);
		add_security_headers(&mut res.headers_mut(), self.safe_to_embed_on);

		(reader, res)
	}
}

type Sender = mpsc::Sender<Result<hyper::Chunk, hyper::Error>>;

const MAX_CHUNK_SIZE: usize = 16 * 1024;
pub struct Reading<R: io::Read> {
	buffer: [u8; MAX_CHUNK_SIZE],
	content: io::BufReader<R>,
	sending: sink::Send<Sender>,
}

impl<R: io::Read> Future for Reading<R> {
	type Item = ();
	type Error = ();

	fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
		loop {
			let next = try_ready!(self.sending.poll().map_err(|err| {
				warn!(target: "dapps", "Unable to send next chunk: {:?}", err);
			}));

			self.sending = match self.content.read(&mut self.buffer) {
				Ok(0) => return Ok(futures::Async::Ready(())),
				Ok(read) => next.send(Ok(self.buffer[..read].to_vec().into())),
				Err(err) => next.send(Err(hyper::Error::Io(err))),
			}
		}
	}
}
