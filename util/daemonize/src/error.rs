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

	/// "Unable to create new session
	#[fail(display = "Unable to create new session: {}", _0)]
	DetachSession(io::Error),

	/// Unable to resolve group name to group id
	#[fail(display = "Unable to resolve group name to group id")]
	GroupNotFound,

	/// Group option contains NUL
	#[fail(display = "Group option contains NUL")]
	GroupContainsNul,

	/// Unable to set group
	#[fail(display = "Unable to set group: {}", _0)]
	SetGroup(io::Error),

	/// Unable to resolve user name to user id
	#[fail(display = "Unable to resolve user name to user id")]
	UserNotFound,

	/// User option contains NUL
	#[fail(display = "User option contains NUL")]
	UserContainsNul,

	/// Unable to set user
	#[fail(display = "Unable to set user: {}", _0)]
	SetUser(io::Error),

	/// Unable to change directory
	#[fail(display = "Unable to change directory")]
	ChangeDirectory,

	/// pid_file option contains NUL
	#[fail(display = "pid_file option contains NUL")]
	PathContainsNul,

	/// Unable to open pid file
	#[fail(display = "Unable to open pid file, {}", _0)]
	OpenPidfile(io::Error),

	/// Unable to lock pid file
	#[fail(display = "Unable to lock pid file: {}", _0)]
	LockPidfile(io::Error),

	/// Unable to chown pid file
	#[fail(display = "Unable to chown pid file: {}", _0)]
	ChownPidfile(io::Error),

	/// Unable to redirect standard streams to /dev/null
	#[fail(display = "Unable to redirect standard streams to /dev/null: {}", _0)]
	RedirectStreams(io::Error),

	/// Unable to write self pid to pid file
	#[fail(display = "Unable to write self pid to pid file {}", _0)]
	WritePid(io::Error),

	/// Unable to chroot
	#[fail(display = "Unable to chroot: {}", _0)]
	Chroot(io::Error),

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
