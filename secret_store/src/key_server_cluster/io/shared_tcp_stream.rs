use std::sync::Arc;
use std::net::Shutdown;
use std::io::{Read, Write, Error};
use tokio_core::net::TcpStream;

pub struct SharedTcpStream {
	io: Arc<TcpStream>,
}

impl SharedTcpStream {
	pub fn new(a: Arc<TcpStream>) -> Self {
		SharedTcpStream {
			io: a,
		}
	}

	pub fn shutdown(&self) {
		// error is irrelevant here, the connection is dropped anyway
		let _ = self.io.shutdown(Shutdown::Both);
	}
}

impl From<TcpStream> for SharedTcpStream {
	fn from(a: TcpStream) -> Self {
		SharedTcpStream::new(Arc::new(a))
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
