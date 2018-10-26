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

use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use tempdir::TempDir;

use parity_runtime::{Runtime, TaskExecutor};

use authcodes::AuthCodes;

/// Server with event loop
pub struct Server<T> {
	/// Server
	pub server: T,
	/// RPC Event Loop
	pub event_loop: Runtime,
}

impl<T> Server<T> {
	pub fn new<F>(f: F) -> Server<T> where
		F: FnOnce(TaskExecutor) -> T,
	{
		let event_loop = Runtime::with_thread_count(1);
		let remote = event_loop.raw_executor();

		Server {
			server: f(remote),
			event_loop: event_loop,
		}
	}
}

impl<T> Deref for Server<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.server
	}
}

/// Struct representing authcodes
pub struct GuardedAuthCodes {
	authcodes: AuthCodes,
	_tempdir: TempDir,
	/// The path to the mock authcodes
	pub path: PathBuf,
}

impl GuardedAuthCodes {
	pub fn new() -> Self {
		let tempdir = TempDir::new("").unwrap();
		let path = tempdir.path().join("file");

		GuardedAuthCodes {
			authcodes: AuthCodes::from_file(&path).unwrap(),
			_tempdir: tempdir,
			path,
		}
	}
}

impl Deref for GuardedAuthCodes {
	type Target = AuthCodes;
	fn deref(&self) -> &Self::Target {
		&self.authcodes
	}
}

impl DerefMut for GuardedAuthCodes {
	fn deref_mut(&mut self) -> &mut AuthCodes {
		&mut self.authcodes
	}
}
