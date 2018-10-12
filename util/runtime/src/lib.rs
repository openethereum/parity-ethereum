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

//! Tokio Runtime wrapper.

extern crate futures;
extern crate tokio;

use std::{fmt, thread};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use futures::{future, Future, IntoFuture};
pub use tokio::timer::Delay;
pub use tokio::runtime::{Runtime as TokioRuntime, Builder as TokioRuntimeBuilder, TaskExecutor};

/// Runtime for futures.
///
/// Runs in a separate thread.
pub struct Runtime {
	executor: Executor,
	handle: RuntimeHandle,
}

impl Runtime {
	fn new(runtime_bldr: &mut TokioRuntimeBuilder) -> Self {
		let mut runtime = runtime_bldr
			.build()
			.expect("Building a Tokio runtime will only fail when mio components \
				cannot be initialized (catastrophic)");
		let (stop, stopped) = futures::oneshot();
		let (tx, rx) = mpsc::channel();
		let handle = thread::spawn(move || {
			tx.send(runtime.executor()).expect("Rx is blocking upper thread.");
			runtime.block_on(futures::empty().select(stopped).map(|_| ()).map_err(|_| ()))
				.expect("Tokio runtime should not have unhandled errors.");
		});
		let executor = rx.recv().expect("tx is transfered to a newly spawned thread.");

		Runtime {
			executor: Executor {
				inner: Mode::Tokio(executor),
			},
			handle: RuntimeHandle {
				close: Some(stop),
				handle: Some(handle),
			},
		}
	}

	/// Spawns a new tokio runtime with a default thread count on a background
	/// thread and returns a `Runtime` which can be used to spawn tasks via
	/// its executor.
	pub fn with_default_thread_count() -> Self {
		let mut runtime_bldr = TokioRuntimeBuilder::new();
		Self::new(&mut runtime_bldr)
	}

	/// Spawns a new tokio runtime with a the specified thread count on a
	/// background thread and returns a `Runtime` which can be used to spawn
	/// tasks via its executor.
	pub fn with_thread_count(thread_count: usize) -> Self {
		let mut runtime_bldr = TokioRuntimeBuilder::new();
		runtime_bldr.core_threads(thread_count);

		Self::new(&mut runtime_bldr)
	}

	/// Returns this runtime raw executor.
	///
	/// Deprecated: Exists only to connect with current JSONRPC implementation.
	pub fn raw_executor(&self) -> TaskExecutor {
		if let Mode::Tokio(ref executor) = self.executor.inner {
			executor.clone()
		} else {
			panic!("Runtime is not initialized in Tokio mode.")
		}
	}

	/// Returns runtime executor.
	pub fn executor(&self) -> Executor {
		self.executor.clone()
	}
}

#[derive(Clone)]
enum Mode {
	Tokio(TaskExecutor),
	Sync,
	ThreadPerFuture,
}

impl fmt::Debug for Mode {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		use self::Mode::*;

		match *self {
			Tokio(_) => write!(fmt, "tokio"),
			Sync => write!(fmt, "synchronous"),
			ThreadPerFuture => write!(fmt, "thread per future"),
		}
	}
}

/// Returns a future which runs `f` until `duration` has elapsed, at which
/// time `on_timeout` is run and the future resolves.
fn timeout<F, R, T>(f: F, duration: Duration, on_timeout: T)
	-> impl Future<Item = (), Error = ()> + Send + 'static
where
	T: FnOnce() -> () + Send + 'static,
	F: FnOnce() -> R + Send + 'static,
	R: IntoFuture<Item=(), Error=()> + Send + 'static,
	R::Future: Send + 'static,
{
	let future = future::lazy(f);
	let timeout = Delay::new(Instant::now() + duration)
		.then(move |_| {
			on_timeout();
			Ok(())
		});
	future.select(timeout).then(|_| Ok(()))
}

#[derive(Debug, Clone)]
pub struct Executor {
	inner: Mode,
}

impl Executor {
	/// Executor for existing runtime.
	///
	/// Deprecated: Exists only to connect with current JSONRPC implementation.
	pub fn new(executor: TaskExecutor) -> Self {
		Executor {
			inner: Mode::Tokio(executor),
		}
	}

	/// Synchronous executor, used mostly for tests.
	pub fn new_sync() -> Self {
		Executor {
			inner: Mode::Sync,
		}
	}

	/// Spawns a new thread for each future (use only for tests).
	pub fn new_thread_per_future() -> Self {
		Executor {
			inner: Mode::ThreadPerFuture,
		}
	}

	/// Spawn a future to this runtime
	pub fn spawn<R>(&self, r: R) where
		R: IntoFuture<Item=(), Error=()> + Send + 'static,
		R::Future: Send + 'static,
	{
		match self.inner {
			Mode::Tokio(ref executor) => executor.spawn(r.into_future()),
			Mode::Sync => {
				let _= r.into_future().wait();
			},
			Mode::ThreadPerFuture => {
				thread::spawn(move || {
					let _= r.into_future().wait();
				});
			},
		}
	}

	/// Spawn a new future returned by given closure.
	pub fn spawn_fn<F, R>(&self, f: F) where
		F: FnOnce() -> R + Send + 'static,
		R: IntoFuture<Item=(), Error=()> + Send + 'static,
		R::Future: Send + 'static,
	{
		match self.inner {
			Mode::Tokio(ref executor) => executor.spawn(future::lazy(f)),
			Mode::Sync => {
				let _ = future::lazy(f).wait();
			},
			Mode::ThreadPerFuture => {
				thread::spawn(move || {
					let _= f().into_future().wait();
				});
			},
		}
	}

	/// Spawn a new future and wait for it or for a timeout to occur.
	pub fn spawn_with_timeout<F, R, T>(&self, f: F, duration: Duration, on_timeout: T) where
		T: FnOnce() -> () + Send + 'static,
		F: FnOnce() -> R + Send + 'static,
		R: IntoFuture<Item=(), Error=()> + Send + 'static,
		R::Future: Send + 'static,
	{
		match self.inner {
			Mode::Tokio(ref executor) => {
				executor.spawn(timeout(f, duration, on_timeout))
			},
			Mode::Sync => {
				let _ = timeout(f, duration, on_timeout).wait();
			},
			Mode::ThreadPerFuture => {
				thread::spawn(move || {
					let _ = timeout(f, duration, on_timeout).wait();
				});
			},
		}
	}
}

/// A handle to a runtime. Dropping the handle will cause runtime to shutdown.
pub struct RuntimeHandle {
	close: Option<futures::Complete<()>>,
	handle: Option<thread::JoinHandle<()>>
}

impl From<Runtime> for RuntimeHandle {
	fn from(el: Runtime) -> Self {
		el.handle
	}
}

impl Drop for RuntimeHandle {
	fn drop(&mut self) {
		self.close.take().map(|v| v.send(()));
	}
}

impl RuntimeHandle {
	/// Blocks current thread and waits until the runtime is finished.
	pub fn wait(mut self) -> thread::Result<()> {
		self.handle.take()
			.expect("Handle is taken only in `wait`, `wait` is consuming; qed").join()
	}

	/// Finishes this runtime.
	pub fn close(mut self) {
		let _ = self.close.take()
			.expect("Close is taken only in `close` and `drop`. `close` is consuming; qed")
			.send(());
	}
}
