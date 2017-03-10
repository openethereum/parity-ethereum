use std::io;
use futures::{Poll, Future, Async};
use key_server_cluster::Error;
use key_server_cluster::message::Message;
use key_server_cluster::io::{read_header, ReadHeader, read_payload, ReadPayload};

pub fn read_message<A>(a: A) -> ReadMessage<A> where A: io::Read {
	ReadMessage {
		state: ReadMessageState::ReadHeader(read_header(a)),
	}
}

enum ReadMessageState<A> {
	ReadHeader(ReadHeader<A>),
	ReadPayload(ReadPayload<A>),
	Finished,
}

pub struct ReadMessage<A> {
	state: ReadMessageState<A>,
}

impl<A> Future for ReadMessage<A> where A: io::Read {
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

				let future = read_payload(read, header);
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
