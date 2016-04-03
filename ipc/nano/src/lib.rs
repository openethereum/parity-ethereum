// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! IPC over nanomsg transport

extern crate ethcore_ipc as ipc;
extern crate nanomsg;
#[macro_use] extern crate log;

pub use ipc::*;

use std::sync::*;
use std::io::Write;
use nanomsg::{Socket, Protocol};

pub struct Worker<S> where S: IpcInterface<S> {
	service: Arc<S>,
	sockets: Vec<Socket>,
	method_buf: [u8;2],
}

impl<S> Worker<S> where S: IpcInterface<S> {
	pub fn new(service: Arc<S>) -> Worker<S> {
		Worker::<S> {
			service: service.clone(),
			sockets: Vec::new(),
			method_buf: [0,0]
		}
	}

	pub fn poll(&mut self) {
		for socket in self.sockets.iter_mut() {
			if let Ok(method_sig_len) = socket.nb_read(&mut self.method_buf) {
				if method_sig_len == 2 {
					let result = self.service.dispatch_buf(
						self.method_buf[1] as u16 * 256 + self.method_buf[0] as u16,
						socket);
					if let Err(e) = socket.write(&result) {
						warn!(target: "ipc", "Failed to write response: {:?}", e);
					}
				}
			}
		}
	}
}
