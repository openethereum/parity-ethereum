#![allow(dead_code)] //TODO: remove this after everything is done
//TODO: remove all unwraps
use mio::*;
use hash::*;
use network::connection::{EncryptedConnection};
use network::handshake::Handshake;
use network::Error;
use network::host::*;

pub struct Session {
	pub id: NodeId,
	connection: EncryptedConnection,
}

impl Session { 
	pub fn new(h: Handshake, event_loop: &mut EventLoop<Host>) -> Result<Session, Error> {
		let id = h.id.clone();
		let mut connection = try!(EncryptedConnection::new(h));
		try!(connection.register(event_loop));
		Ok(Session {
			id: id,
			connection: connection,
		})
	}
	pub fn readable(&mut self, event_loop: &mut EventLoop<Host>, _host: &HostInfo) -> Result<(), Error> {
		try!(self.connection.readable(event_loop));
		Ok(())
	}
	pub fn writable(&mut self, event_loop: &mut EventLoop<Host>, _host: &HostInfo) -> Result<(), Error> {
		self.connection.writable(event_loop)
	}
}

