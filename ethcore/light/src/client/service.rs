// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Minimal IO service for light client.
//! Just handles block import messages and passes them to the client.

use std::sync::Arc;

use ethcore::service::ClientIoMessage;
use ethcore::spec::Spec;
use io::{IoContext, IoError, IoHandler, IoService};

use super::{Client, Config as ClientConfig};

/// Light client service.
pub struct Service {
	client: Arc<Client>,
	_io_service: IoService<ClientIoMessage>,
}

impl Service {
	/// Start the service: initialize I/O workers and client itself.
	pub fn start(config: ClientConfig, spec: &Spec) -> Result<Self, IoError> {
		let io_service = try!(IoService::<ClientIoMessage>::start());
		let client = Arc::new(Client::new(config, spec, io_service.channel()));
		try!(io_service.register_handler(Arc::new(ImportBlocks(client.clone()))));

		Ok(Service {
			client: client,
			_io_service: io_service,
		})
	}

	/// Get a handle to the client.
	pub fn client(&self) -> &Arc<Client> {
		&self.client
	}
}

struct ImportBlocks(Arc<Client>);

impl IoHandler<ClientIoMessage> for ImportBlocks {
	fn message(&self, _io: &IoContext<ClientIoMessage>, message: &ClientIoMessage) {
		if let ClientIoMessage::BlockVerified = *message {
			self.0.import_verified();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Service;
	use ethcore::spec::Spec;

	#[test]
	fn it_works() {
		let spec = Spec::new_test();
		Service::start(Default::default(), &spec).unwrap();
	}
}
