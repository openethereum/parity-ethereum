use std::io;
use std::marker::PhantomData;
use futures::{Poll, Future};
use tokio_core::io::{read_exact, ReadExact};
use key_server_cluster::Error;
use key_server_cluster::message::Message;
use key_server_cluster::io::message::{MessageHeader, deserialize_message};

pub fn read_payload<A>(a: A, header: MessageHeader) -> ReadPayload<A> where A: io::Read {
	ReadPayload {
		reader: read_exact(a, vec![0; header.size as usize]),
		header: header,
	}
}

pub struct ReadPayload<A> {
	reader: ReadExact<A, Vec<u8>>,
	header: MessageHeader,
}

impl<A> Future for ReadPayload<A> where A: io::Read {
	type Item = (A, Result<Message, Error>);
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let (read, data) = try_ready!(self.reader.poll());
		let payload = deserialize_message(&self.header, data);
		Ok((read, payload).into())
	}
}
