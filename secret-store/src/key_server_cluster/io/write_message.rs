// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use ethkey::KeyPair;
use futures::{Future, Poll};
use key_server_cluster::{
    io::{encrypt_message, serialize_message},
    message::Message,
};
use std::io;
use tokio_io::{
    io::{write_all, WriteAll},
    AsyncWrite,
};

/// Write plain message to the channel.
pub fn write_message<A>(a: A, message: Message) -> WriteMessage<A>
where
    A: AsyncWrite,
{
    let (error, future) = match serialize_message(message)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    {
        Ok(message) => (None, write_all(a, message.into())),
        Err(error) => (Some(error), write_all(a, Vec::new())),
    };
    WriteMessage {
        error: error,
        future: future,
    }
}

/// Write encrypted message to the channel.
pub fn write_encrypted_message<A>(a: A, key: &KeyPair, message: Message) -> WriteMessage<A>
where
    A: AsyncWrite,
{
    let (error, future) = match serialize_message(message)
        .and_then(|message| encrypt_message(key, message))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    {
        Ok(message) => (None, write_all(a, message.into())),
        Err(error) => (Some(error), write_all(a, Vec::new())),
    };

    WriteMessage {
        error: error,
        future: future,
    }
}

/// Future message write.
pub struct WriteMessage<A> {
    error: Option<io::Error>,
    future: WriteAll<A, Vec<u8>>,
}

impl<A> Future for WriteMessage<A>
where
    A: AsyncWrite,
{
    type Item = (A, Vec<u8>);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(err) = self.error.take() {
            return Err(err);
        }

        self.future.poll()
    }
}
