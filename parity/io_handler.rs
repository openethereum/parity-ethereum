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

use std::sync::Arc;
use ethcore::client::Client;
use ethcore::service::ClientIoMessage;
use ethsync::{EthSync, SyncProvider, ManageNetwork};
use ethcore::account_provider::AccountProvider;
use util::{TimerToken, IoHandler, IoContext};

use informant::Informant;

const INFO_TIMER: TimerToken = 0;

pub struct ClientIoHandler {
	pub client: Arc<Client>,
	pub sync: Arc<EthSync>,
	pub accounts: Arc<AccountProvider>,
	pub info: Informant,
}

impl IoHandler<ClientIoMessage> for ClientIoHandler {
	fn initialize(&self, io: &IoContext<ClientIoMessage>) {
		io.register_timer(INFO_TIMER, 5000).expect("Error registering timer");
	}

	fn timeout(&self, _io: &IoContext<ClientIoMessage>, timer: TimerToken) {
		if let INFO_TIMER = timer {
			let sync_status = self.sync.status();
			let network_config = self.sync.network_config();
			self.info.tick(&self.client, Some((sync_status, network_config)));
		}
	}
}
