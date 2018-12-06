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
use failure::{Backtrace, Context, Fail};
use std::fmt::{self, Display};
use std::io;

/// Error type
#[derive(Debug)]
pub struct Error {
	inner: Context<ErrorKind>,
}

/// Possible errors encountered while creating daemon
#[derive(Fail, Debug)]
pub enum ErrorKind {
	/// Call to pipe failed
	#[fail(display = "Call to pipe failed: {}", _0)]
	Pipe(io::Error),

	/// Couldn't fork daemon process
	#[fail(display = "Couldn't fork daemon process: {}", _0)]
	Fork(io::Error),

	/// Couldn't redirect stdio streams
	#[fail(display = "Couldn't redirect stdio streams: {}", _0)]
	Dup2(io::Error),

	/// Unable to create new session
	#[fail(display = "Unable to create new session: {}", _0)]
	DetachSession(io::Error),

	/// Unable to change directory
	#[fail(display = "Unable to change directory")]
	ChangeDirectory,

	/// pid_file option contains NUL
	#[fail(display = "pid_file option contains NUL")]
	PathContainsNul,

	/// Unable to open pid file
	#[fail(display = "Unable to open pid file, {}", _0)]
	OpenPidfile(io::Error),

	/// Unable to write self pid to pid file
	#[fail(display = "Unable to write self pid to pid file {}", _0)]
	WritePid(io::Error),

	/// failed to register pipe fd's with mio
	#[fail(display = "Unable to register pipe with mio: {}", _0)]
	RegisterationError(io::Error),

	/// splice returned an error
	#[fail(display = "Unable to splice streams: {}", _0)]
	SpliceError(io::Error),

	/// couldn't get the pending datasize from ioctl
	#[fail(display = "Unable to fetch pending datasize from ioctl: {}", _0)]
	Ioctl(io::Error),

	/// attempted to daemonize in windows
	#[fail(display = "Windows doesn't support daemons")]
	Windows
}

impl Fail for Error {
	fn cause(&self) -> Option<&Fail> {
		self.inner.cause()
	}

	fn backtrace(&self) -> Option<&Backtrace> {
		self.inner.backtrace()
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		Display::fmt(&self.inner, f)
	}
}

impl Error {
	/// extract the error kind
	pub fn kind(&self) -> &ErrorKind {
		self.inner.get_context()
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Error {
		Error {
			inner: Context::new(kind),
		}
	}
}

impl From<Context<ErrorKind>> for Error {
	fn from(inner: Context<ErrorKind>) -> Error {
		Error { inner }
	}
}
