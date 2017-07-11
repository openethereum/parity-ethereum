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

//! Async Content Handler
//! Temporary solution until we switch to future-based server.
//! Wraps a future and converts it to hyper::server::Handler;

use std::{mem, time};
use std::sync::mpsc;
use futures::Future;
use hyper::{server, Decoder, Encoder, Next, Control};
use hyper::net::HttpStream;

use handlers::ContentHandler;
use parity_reactor::Remote;

const TIMEOUT_SECS: u64 = 15;

enum State<F, T, M> {
	Initial(F, M, Remote, Control),
	Waiting(mpsc::Receiver<Result<T, ()>>, M),
	Done(ContentHandler),
	Invalid,
}

pub struct AsyncHandler<F, T, M> {
	state: State<F, T, M>,
}

impl<F, T, M> AsyncHandler<F, T, M> {
	pub fn new(future: F, map: M, remote: Remote, control: Control) -> Self {
		AsyncHandler {
			state: State::Initial(future, map, remote, control),
		}
	}
}

impl<F, T, E, M> server::Handler<HttpStream> for AsyncHandler<F, Result<T, E>, M> where
	F: Future<Item=T, Error=E> + Send + 'static,
	M: FnOnce(Result<Result<T, E>, ()>) -> ContentHandler,
	T: Send + 'static,
	E: Send + 'static,
{
	fn on_request(&mut self, _request: server::Request<HttpStream>) -> Next {
		if let State::Initial(future, map, remote, control) = mem::replace(&mut self.state, State::Invalid) {
			let (tx, rx) = mpsc::sync_channel(1);
			let control2 = control.clone();
			let tx2 = tx.clone();
			remote.spawn_with_timeout(move || future.then(move |result| {
				// Send a result (ignore errors if the connection was dropped)
				let _ = tx.send(Ok(result));
				// Resume handler
				let _ = control.ready(Next::read());

				Ok(())
			}), time::Duration::from_secs(TIMEOUT_SECS), move || {
				// Notify about error
				let _ = tx2.send(Err(()));
				// Resume handler
				let _ = control2.ready(Next::read());
			});

			self.state = State::Waiting(rx, map);
		}

		Next::wait()
	}

	fn on_request_readable(&mut self, _decoder: &mut Decoder<HttpStream>) -> Next {
		if let State::Waiting(rx, map) = mem::replace(&mut self.state, State::Invalid) {
			match rx.try_recv() {
				Ok(result) => {
					self.state = State::Done(map(result));
				},
				Err(err) => {
					warn!("Resuming handler in incorrect state: {:?}", err);
				}
			}
		}

		Next::write()
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		if let State::Done(ref mut handler) = self.state {
			handler.on_response(res)
		} else {
			Next::end()
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<HttpStream>) -> Next {
		if let State::Done(ref mut handler) = self.state {
			handler.on_response_writable(encoder)
		} else {
			Next::end()
		}
	}
}
