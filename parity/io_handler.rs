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
use ethcore::service::{NetSyncMessage, SyncMessage};
use ethsync::EthSync;
use util::keys::store::AccountService;
use util::{TimerToken, IoHandler, IoContext, NetworkService, NetworkIoMessage};

use informant::Informant;

const INFO_TIMER: TimerToken = 0;

const ACCOUNT_TICK_TIMER: TimerToken = 10;
const ACCOUNT_TICK_MS: u64 = 60000;

pub struct ClientIoHandler {
	pub client: Arc<Client>,
	pub sync: Arc<EthSync>,
	pub accounts: Arc<AccountService>,
	pub info: Informant,
	pub network: Arc<NetworkService<SyncMessage>>,
}

impl IoHandler<NetSyncMessage> for ClientIoHandler {
	fn initialize(&self, io: &IoContext<NetSyncMessage>) {
		io.register_timer(INFO_TIMER, 5000).expect("Error registering timer");
		io.register_timer(ACCOUNT_TICK_TIMER, ACCOUNT_TICK_MS).expect("Error registering account timer");
	}

	fn timeout(&self, _io: &IoContext<NetSyncMessage>, timer: TimerToken) {
		match timer {
			INFO_TIMER => { self.info.tick(&self.client, Some(&self.sync)); }
			ACCOUNT_TICK_TIMER => { self.accounts.tick(); },
			_ => {}
		}
	}

	fn message(&self, _io: &IoContext<NetSyncMessage>, message: &NetSyncMessage) {
		match *message {
			NetworkIoMessage::User(SyncMessage::StartNetwork) => {
				info!("Starting network");
				self.network.start().unwrap_or_else(|e| warn!("Error starting network: {:?}", e));
				EthSync::register(&*self.network, self.sync.clone()).unwrap_or_else(|e| warn!("Error registering eth protocol handler: {}", e));
			},
			NetworkIoMessage::User(SyncMessage::StopNetwork) => {
				info!("Stopping network");
				self.network.stop().unwrap_or_else(|e| warn!("Error stopping network: {:?}", e));
			},
			_ => {/* Ignore other messages */},
		}
	}
}


