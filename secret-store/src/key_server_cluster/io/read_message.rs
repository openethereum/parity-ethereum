// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::io;
use futures::{Poll, Future, Async};
use tokio_io::AsyncRead;
use crypto::publickey::KeyPair;
use key_server_cluster::Error;
use key_server_cluster::message::Message;
use key_server_cluster::io::{read_header, ReadHeader, read_payload, read_encrypted_payload, ReadPayload};

/// Create future for read single message from the stream.
pub fn read_message<A>(a: A) -> ReadMessage<A> where A: AsyncRead {
	ReadMessage {
		key: None,
		state: ReadMessageState::ReadHeader(read_header(a)),
	}
}

/// Create future for read single encrypted message from the stream.
pub fn read_encrypted_message<A>(a: A, key: KeyPair) -> ReadMessage<A> where A: AsyncRead {
	ReadMessage {
		key: Some(key),
		state: ReadMessageState::ReadHeader(read_header(a)),
	}
}

enum ReadMessageState<A> {
	ReadHeader(ReadHeader<A>),
	ReadPayload(ReadPayload<A>),
	Finished,
}

/// Future for read single message from the stream.
pub struct ReadMessage<A> {
	key: Option<KeyPair>,
	state: ReadMessageState<A>,
}

impl<A> Future for ReadMessage<A> where A: AsyncRead {
	type Item = (A, Result<Message, Error>);
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let (next, result) = match self.state {
			ReadMessageState::ReadHeader(ref mut future) => {
				let (read, header) = try_ready!(future.poll());
				let header = match header {
					Ok(header) => header,
					Err(err) => return Ok((read, Err(err)).into()),
				};

				let future = match self.key.take() {
					Some(key) => read_encrypted_payload(read, header, key),
					None => read_payload(read, header),
				};
				let next = ReadMessageState::ReadPayload(future);
				(next, Async::NotReady)
			},
			ReadMessageState::ReadPayload(ref mut future) => {
				let (read, payload) = try_ready!(future.poll());
				(ReadMessageState::Finished, Async::Ready((read, payload)))
			},
			ReadMessageState::Finished => panic!("poll ReadMessage after it's done"),
		};

		self.state = next;
		match result {
			// by polling again, we register new future
			Async::NotReady => self.poll(),
			result => Ok(result)
		}
	}
}
