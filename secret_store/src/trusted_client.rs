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

use std::sync::{Arc, Weak};
use ethcore::client::{Client, BlockChainClient, ChainInfo};
use ethsync::SyncProvider;

#[derive(Clone)]
/// 'Trusted' client weak reference.
pub struct TrustedClient {
	/// Blockchain client.
	client: Weak<Client>,
	/// Sync provider.
	sync: Weak<SyncProvider>,
}

impl TrustedClient {
	/// Create new trusted client.
	pub fn new(client: Arc<Client>, sync: Arc<SyncProvider>) -> Self {
		TrustedClient {
			client: Arc::downgrade(&client),
			sync: Arc::downgrade(&sync),
		}
	}

	/// Get 'trusted' `Client` reference only if it is synchronized && trusted.
	pub fn get(&self) -> Option<Arc<Client>> {
		self.client.upgrade()
			.and_then(|client| self.sync.upgrade().map(|sync| (client, sync)))
			.and_then(|(client, sync)| {
				let is_synced = !sync.status().is_syncing(client.queue_info());
				let is_trusted = client.chain_info().security_level().is_full();
				match is_synced && is_trusted {
					true => Some(client),
					false => None,
				}
			})
	}

	/// Get untrusted `Client` reference.
	pub fn get_untrusted(&self) -> Option<Arc<Client>> {
		self.client.upgrade()
	}
}
