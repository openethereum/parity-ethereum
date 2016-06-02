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

//! Session handlers factory.

use ws;
use std::sync::Arc;
use jsonrpc_core::IoHandler;

pub struct Session {
	out: ws::Sender,
	handler: Arc<IoHandler>,
}

impl ws::Handler for Session {
	fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
		let req = try!(msg.as_text());
		match self.handler.handle_request(req) {
			Some(res) => self.out.send(res),
			None => Ok(()),
		}
	}
}

pub struct Factory {
	handler: Arc<IoHandler>,
}

impl Factory {
	pub fn new(handler: Arc<IoHandler>) -> Self {
		Factory {
			handler: handler,
		}
	}
}

impl ws::Factory for Factory {
	type Handler = Session;

	fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
		Session {
			out: sender,
			handler: self.handler.clone(),
		}
	}
}
