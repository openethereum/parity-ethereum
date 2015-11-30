use std::io::{self, Cursor, Read};
use mio::*;
use mio::tcp::*;
use hash::*;
use bytes::*;
use network::host::Host;

pub struct Connection {
	pub token: Token,
    pub socket: TcpStream,
	rec_buf: Bytes,
	rec_size: usize,
	send_buf: Cursor<Bytes>,
	interest: EventSet,
}

pub enum WriteStatus {
	Ongoing,
	Complete
}

impl Connection {
	pub fn new(token: Token, socket: TcpStream) -> Connection {
		Connection {
			token: token,
			socket: socket,
			send_buf: Cursor::new(Bytes::new()),
			rec_buf: Bytes::new(),
			rec_size: 0,
			interest: EventSet::hup(),
		}
	}

	pub fn expect(&mut self, size: usize) {
		if self.rec_size != self.rec_buf.len() {
			warn!(target:"net", "Unexpected connection read start");
		}
		unsafe { self.rec_buf.set_len(size) }
		self.rec_size = size;
	}

	pub fn readable(&mut self) -> io::Result<Option<&[u8]>> {
		if self.rec_size == 0 || self.rec_buf.len() >= self.rec_size {
			warn!(target:"net", "Unexpected connection read");
		}
		let max = self.rec_size - self.rec_buf.len();
		// resolve "multiple applicable items in scope [E0034]" error
    	let sock_ref = <TcpStream as Read>::by_ref(&mut self.socket);
		match sock_ref.take(max as u64).try_read_buf(&mut self.rec_buf) {
			Ok(Some(_)) if self.rec_buf.len() == self.rec_size => Ok(Some(&self.rec_buf[0..self.rec_size])),
			Ok(_) => Ok(None),
			Err(e) => Err(e),
		}
	}
	
	pub fn send(&mut self, data: &[u8]) { //TODO: take ownership version
		let send_size = self.send_buf.get_ref().len();
		if send_size != 0 || self.send_buf.position() as usize >= send_size {
			warn!(target:"net", "Unexpected connection send start");
		}
		if self.send_buf.get_ref().capacity() < data.len() {
			let capacity = self.send_buf.get_ref().capacity();
			self.send_buf.get_mut().reserve(data.len() - capacity);
		}
		unsafe { self.send_buf.get_mut().set_len(data.len()) }
		unsafe { ::std::ptr::copy_nonoverlapping(data.as_ptr(), self.send_buf.get_mut()[..].as_mut_ptr(), data.len()) };
        if !self.interest.is_writable() {
            self.interest.insert(EventSet::writable());
        }
	}

	pub fn writable(&mut self) -> io::Result<WriteStatus> {
		let send_size = self.send_buf.get_ref().len();
		if (self.send_buf.position() as usize) >= send_size {
			warn!(target:"net", "Unexpected connection data");
			return Ok(WriteStatus::Complete)
		}
		match self.socket.try_write_buf(&mut self.send_buf) {
			Ok(_) if (self.send_buf.position() as usize) < send_size => {
				self.interest.insert(EventSet::writable());
				Ok(WriteStatus::Ongoing)
			},
			Ok(_) if (self.send_buf.position() as usize) == send_size => {
				self.interest.remove(EventSet::writable());
				Ok(WriteStatus::Complete)
			},
			Ok(_) => { panic!("Wrote past buffer");},
			Err(e) => Err(e)
		}
	}

    pub fn register(&mut self, event_loop: &mut EventLoop<Host>) -> io::Result<()> {
        trace!(target: "net", "connection register; token={:?}", self.token);
        self.interest.insert(EventSet::readable());
        event_loop.register_opt(&self.socket, self.token, self.interest, PollOpt::edge() | PollOpt::oneshot()).or_else(|e| {
            error!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }

    pub fn reregister(&mut self, event_loop: &mut EventLoop<Host>) -> io::Result<()> {
        trace!(target: "net", "connection reregister; token={:?}", self.token);
        event_loop.reregister( &self.socket, self.token, self.interest, PollOpt::edge() | PollOpt::oneshot()).or_else(|e| {
            error!("Failed to reregister {:?}, {:?}", self.token, e);
            Err(e)
        })
    }
}

