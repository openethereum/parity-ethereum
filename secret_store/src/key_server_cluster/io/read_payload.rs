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
use futures::{Poll, Future};
use tokio_io::AsyncRead;
use tokio_io::io::{read_exact, ReadExact};
use ethkey::KeyPair;
use key_server_cluster::Error;
use key_server_cluster::message::Message;
use key_server_cluster::io::message::{MessageHeader, deserialize_message, decrypt_message};

/// Create future for read single message payload from the stream.
pub fn read_payload<A>(a: A, header: MessageHeader) -> ReadPayload<A> where A: AsyncRead {
	ReadPayload {
		reader: read_exact(a, vec![0; header.size as usize]),
		header: header,
		key: None,
	}
}

/// Create future for read single encrypted message payload from the stream.
pub fn read_encrypted_payload<A>(a: A, header: MessageHeader, key: KeyPair) -> ReadPayload<A> where A: AsyncRead {
	ReadPayload {
		reader: read_exact(a, vec![0; header.size as usize]),
		header: header,
		key: Some(key),
	}
}

/// Future for read single message payload from the stream.
pub struct ReadPayload<A> {
	reader: ReadExact<A, Vec<u8>>,
	header: MessageHeader,
	key: Option<KeyPair>,
}

impl<A> Future for ReadPayload<A> where A: AsyncRead {
	type Item = (A, Result<Message, Error>);
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let (read, data) = try_ready!(self.reader.poll());
		let payload = if let Some(key) = self.key.take() {
			decrypt_message(&key, data)
				.and_then(|data| deserialize_message(&self.header, data))
		} else {
			deserialize_message(&self.header, data)
		};
		Ok((read, payload).into())
	}
}
