use std::io;
use futures::{Future, Poll, Async};
use tokio_core::io::{ReadExact, read_exact};
use key_server_cluster::Error;
use key_server_cluster::io::message::{MESSAGE_HEADER_SIZE, MessageHeader, deserialize_header};

pub fn read_header<A>(a: A) -> ReadHeader<A> where A: io::Read {
	ReadHeader {
		reader: read_exact(a, vec![0; MESSAGE_HEADER_SIZE]),
	}
}

pub struct ReadHeader<A> {
	reader: ReadExact<A, Vec<u8>>,
}

impl<A> Future for ReadHeader<A> where A: io::Read {
	type Item = (A, Result<MessageHeader, Error>);
	type Error = io::Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let (read, data) = try_ready!(self.reader.poll());
		let header = deserialize_header(data);
		Ok(Async::Ready((read, header)))
	}
}
