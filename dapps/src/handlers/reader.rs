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

//! A chunk-producing io::Read wrapper.

use std::io::{self, Read};

use futures::{self, sink, Sink, Future};
use futures::sync::mpsc;
use hyper;

type Sender = mpsc::Sender<Result<hyper::Chunk, hyper::Error>>;

const MAX_CHUNK_SIZE: usize = 32 * 1024;

/// A Reader is essentially a stream of `hyper::Chunks`.
/// The chunks are read from given `io::Read` instance.
///
/// Unfortunately `hyper` doesn't allow you to pass `Stream`
/// directly to the response, so you need to create
/// a `Body::pair()` and send over chunks using `sink::Send`.
/// Also `Chunks` need to take `Vec` by value, so we need
/// to allocate it for each chunk being sent.
pub struct Reader<R: io::Read> {
	buffer: [u8; MAX_CHUNK_SIZE],
	content: io::BufReader<R>,
	sending: sink::Send<Sender>,
}

impl<R: io::Read> Reader<R> {
	pub fn pair(content: R, initial: Vec<u8>) -> (Self, hyper::Body) {
		let (tx, rx) = hyper::Body::pair();
		let reader = Reader {
			buffer: [0; MAX_CHUNK_SIZE],
			content: io::BufReader::new(content),
			sending: tx.send(Ok(initial.into())),
		};

		(reader, rx)
	}
}

impl<R: io::Read> Future for Reader<R> {
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
