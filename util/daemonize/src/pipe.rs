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

use std::os::unix::io::{FromRawFd, RawFd, AsRawFd};
use std::{fs, io};

#[derive(Debug)]
pub struct Io {
	file: fs::File,
}

impl Io {
	pub fn from_fd(fd: RawFd) -> io::Result<Self> {
		let file = unsafe {
			let previous = libc::fcntl(fd, libc::F_GETFL);

			if previous < 0 {
				return Err(io::Error::last_os_error());
			}

			if libc::fcntl(fd, libc::F_SETFL, previous | libc::O_NONBLOCK) < 0 {
				return Err(io::Error::last_os_error());
			}

			Io {
				file: fs::File::from_raw_fd(fd),
			}
		};
		Ok(file)
	}
}

impl mio::Evented for Io {
	fn register(
		&self,
		poll: &mio::Poll,
		token: mio::Token,
		interest: mio::Ready,
		opts: mio::PollOpt,
	) -> io::Result<()> {
		mio::unix::EventedFd(&self.file.as_raw_fd()).register(poll, token, interest, opts)
	}

	fn reregister(
		&self,
		poll: &mio::Poll,
		token: mio::Token,
		interest: mio::Ready,
		opts: mio::PollOpt,
	) -> io::Result<()> {
		mio::unix::EventedFd(&self.file.as_raw_fd()).reregister(poll, token, interest, opts)
	}

	fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
		mio::unix::EventedFd(&self.file.as_raw_fd()).deregister(poll)
	}
}

impl io::Read for Io {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.file.read(buf)
	}
}

impl io::Write for Io {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.file.write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.file.flush()
	}
}

impl io::Seek for Io {
	fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
		self.file.seek(pos)
	}
}
