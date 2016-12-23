// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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


//! Tokio Core Reactor wrapper.

extern crate futures;
extern crate tokio_core;

use std::thread;
use std::sync::mpsc;
use futures::{Future, IntoFuture};
use self::tokio_core::reactor::Remote as TokioRemote;

/// Event Loop for futures.
/// Wrapper around `tokio::reactor::Core`.
/// Runs in a separate thread.
pub struct EventLoop {
	remote: Remote,
	handle: EventLoopHandle,
}

impl EventLoop {
	/// Spawns a new thread with `EventLoop` with given handler.
	pub fn spawn() -> Self {
		let (stop, stopped) = futures::oneshot();
		let (tx, rx) = mpsc::channel();
		let handle = thread::spawn(move || {
			let mut el = tokio_core::reactor::Core::new().expect("Creating an event loop should not fail.");
			tx.send(el.remote()).expect("Rx is blocking upper thread.");
			let _ = el.run(futures::empty().select(stopped));
		});
		let remote = rx.recv().expect("tx is transfered to a newly spawned thread.");

		EventLoop {
			remote: Remote{
				inner: Mode::Tokio(remote),
			},
			handle: EventLoopHandle {
				close: Some(stop),
				handle: Some(handle),
			},
		}
	}

	/// Returns this event loop raw remote.
	///
	/// Deprecated: Exists only to connect with current JSONRPC implementation.
	pub fn raw_remote(&self) -> TokioRemote {
		if let Mode::Tokio(ref remote) = self.remote.inner {
			remote.clone()
		} else {
			panic!("Event loop is never initialized in other mode then Tokio.")
		}
	}

	/// Returns event loop remote.
	pub fn remote(&self) -> Remote {
		self.remote.clone()
	}
}

#[derive(Clone)]
enum Mode {
	Tokio(TokioRemote),
	Sync,
}

#[derive(Clone)]
pub struct Remote {
	inner: Mode,
}

impl Remote {
	/// Remote for existing event loop.
	///
	/// Deprecated: Exists only to connect with current JSONRPC implementation.
	pub fn new(remote: TokioRemote) -> Self {
		Remote {
			inner: Mode::Tokio(remote),
		}
	}

	/// Synchronous remote, used mostly for tests.
	pub fn new_sync() -> Self {
		Remote {
			inner: Mode::Sync,
		}
	}

	/// Spawn a future to this event loop
	pub fn spawn<R>(&self, r: R) where
        R: IntoFuture<Item=(), Error=()> + Send + 'static,
        R::Future: 'static,
	{
		match self.inner {
			Mode::Tokio(ref remote) => remote.spawn(move |_| r),
			Mode::Sync => {
				let _= r.into_future().wait();
			},
		}
	}

	/// Spawn a new future returned by given closure.
	pub fn spawn_fn<F, R>(&self, f: F) where
		F: FnOnce() -> R + Send + 'static,
        R: IntoFuture<Item=(), Error=()>,
        R::Future: 'static,
	{
		match self.inner {
			Mode::Tokio(ref remote) => remote.spawn(move |_| f()),
			Mode::Sync => {
				let _ = f().into_future().wait();
			},
		}
	}
}

/// A handle to running event loop. Dropping the handle will cause event loop to finish.
pub struct EventLoopHandle {
	close: Option<futures::Complete<()>>,
	handle: Option<thread::JoinHandle<()>>
}

impl From<EventLoop> for EventLoopHandle {
	fn from(el: EventLoop) -> Self {
		el.handle
	}
}

impl Drop for EventLoopHandle {
	fn drop(&mut self) {
		self.close.take().map(|v| v.complete(()));
	}
}

impl EventLoopHandle {
	/// Blocks current thread and waits until the event loop is finished.
	pub fn wait(mut self) -> thread::Result<()> {
		self.handle.take()
			.expect("Handle is taken only in `wait`, `wait` is consuming; qed").join()
	}

	/// Finishes this event loop.
	pub fn close(mut self) {
		self.close.take()
			.expect("Close is taken only in `close` and `drop`. `close` is consuming; qed").complete(())
	}
}
