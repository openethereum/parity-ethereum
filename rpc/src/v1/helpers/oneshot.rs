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

use jsonrpc_core::Error;
use jsonrpc_core::futures::{self, Future};
use jsonrpc_core::futures::sync::oneshot;
use v1::helpers::errors;

pub type Res<T> = Result<T, Error>;

pub struct Sender<T> {
	sender: oneshot::Sender<Res<T>>,
}

impl<T> Sender<T> {
	pub fn send(self, data: Res<T>) {
		let res = self.sender.send(data);
		if let Err(_) = res {
			debug!(target: "rpc", "Responding to a no longer active request.");
		}
	}
}

pub struct Receiver<T> {
	receiver: oneshot::Receiver<Res<T>>,
}

impl<T> Future for Receiver<T> {
	type Item = T;
	type Error = Error;

	fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
		let res = self.receiver.poll();
		match res {
			Ok(futures::Async::NotReady) => Ok(futures::Async::NotReady),
			Ok(futures::Async::Ready(Ok(res))) => Ok(futures::Async::Ready(res)),
			Ok(futures::Async::Ready(Err(err))) => Err(err),
			Err(e) => {
				debug!(target: "rpc", "Responding to a canceled request: {:?}", e);
				Err(errors::internal("Request was canceled by client.", e))
			},
		}
	}
}

pub fn oneshot<T>() -> (Sender<T>, Receiver<T>) {
	let (tx, rx) = futures::oneshot();

	(Sender {
		sender: tx,
	}, Receiver {
		receiver: rx,
	})
}
