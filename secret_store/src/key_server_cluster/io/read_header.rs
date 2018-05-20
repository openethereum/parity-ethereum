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

use std::io;
use futures::{Future, Poll, Async};
use tokio_io::AsyncRead;
use tokio_io::io::{ReadExact, read_exact};
use key_server_cluster::Error;
use key_server_cluster::io::message::{MESSAGE_HEADER_SIZE, MessageHeader, deserialize_header};

/// Create future for read single message header from the stream.
pub fn read_header<A>(a: A) -> ReadHeader<A> where A: AsyncRead {
	ReadHeader {
		reader: read_exact(a, [0; MESSAGE_HEADER_SIZE]),
	}
}

/// Future for read single message header from the stream.
pub struct ReadHeader<A> {
	reader: ReadExact<A, [u8; MESSAGE_HEADER_SIZE]>,
}

impl<A> Future for ReadHeader<A> where A: AsyncRead {
	type Item = (A, Result<MessageHeader, Error>);
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let (read, data) = try_ready!(self.reader.poll());
		let header = deserialize_header(&data);
		Ok(Async::Ready((read, header)))
	}
}
