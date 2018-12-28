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

use std::sync::Arc;
use std::net::Shutdown;
use std::io::{Read, Write, Error};
use futures::Poll;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

/// Read+Write implementation for Arc<TcpStream>.
pub struct SharedTcpStream {
	io: Arc<TcpStream>,
}

impl SharedTcpStream {
	pub fn new(a: Arc<TcpStream>) -> Self {
		SharedTcpStream {
			io: a,
		}
	}
}

impl From<TcpStream> for SharedTcpStream {
	fn from(a: TcpStream) -> Self {
		SharedTcpStream::new(Arc::new(a))
	}
}

impl AsyncRead for SharedTcpStream {}

impl AsyncWrite for SharedTcpStream {
	fn shutdown(&mut self) -> Poll<(), Error> {
		self.io.shutdown(Shutdown::Both).map(Into::into)
	}
}

impl Read for SharedTcpStream {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
		Read::read(&mut (&*self.io as &TcpStream), buf)
	}
}

impl Write for SharedTcpStream {
	fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
		Write::write(&mut (&*self.io as &TcpStream), buf)
	}

	fn flush(&mut self) -> Result<(), Error> {
		Write::flush(&mut (&*self.io as &TcpStream))
	}
}

impl Clone for SharedTcpStream {
	fn clone(&self) -> Self {
		SharedTcpStream::new(self.io.clone())
	}
}
