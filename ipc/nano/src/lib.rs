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

pub use ipc::*;

use std::sync::*;
use nanomsg::{Socket, Protocol};

pub struct Worker<S> where S: IpcInterface<S> {
	service: Arc<S>,
	sockets: Vec<Socket>,
}

impl<S> Worker<S> where S: IpcInterface<S> {
	pub fn new(service: Arc<S>, socket_addr: &str) -> Worker<S> {
		Worker::<S> {
			service: service.clone(),
			sockets: Vec::new(),
		}
	}

	pub fn work_loop(&mut self) {
	}
}
