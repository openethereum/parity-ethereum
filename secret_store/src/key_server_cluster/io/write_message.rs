use std::io;
use futures::{Future, Poll};
use tokio_core::io::{WriteAll, write_all};
use ethkey::Public;
use key_server_cluster::Error;
use key_server_cluster::message::Message;
use key_server_cluster::io::{serialize_message, encrypt_message};

/// Write plain message to the channel.
pub fn write_message<A>(a: A, message: Message) -> WriteMessage<A> where A: io::Write {
	let message = serialize_message(message).unwrap(); // TODO
	WriteMessage {
		future: write_all(a, message.into()),
	}
}

/// Write encrypted message to the channel.
pub fn write_encrypted_message<A>(a: A, key: &Public, message: Message) -> WriteMessage<A> where A: io::Write {
	let message = serialize_message(message).unwrap(); // TODO
	let message = encrypt_message(key, message).unwrap(); // TODO
	WriteMessage {
		future: write_all(a, message.into()),
	}
}

/// Future message write.
pub struct WriteMessage<A> {
	future: WriteAll<A, Vec<u8>>,
}

impl<A> Future for WriteMessage<A> where A: io::Write {
	type Item = (A, Vec<u8>);
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		self.future.poll()
	}
}
