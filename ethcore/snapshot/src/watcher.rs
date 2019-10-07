// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Watcher for snapshot-related chain events.

use std::sync::Arc;

use client_traits::{BlockInfo, ChainNotify};
use common_types::{
	ids::BlockId,
	io_message::ClientIoMessage,
	chain_notify::NewBlocks,
};
use ethereum_types::H256;
use ethcore_io::IoChannel;
use log::{trace, warn};
use parking_lot::Mutex;

use crate::traits::{Broadcast, Oracle};

struct StandardOracle<F> where F: 'static + Send + Sync + Fn() -> bool {
	client: Arc<dyn BlockInfo>,
	sync_status: F,
}

impl<F> Oracle for StandardOracle<F>
	where F: Send + Sync + Fn() -> bool
{
	fn to_number(&self, hash: H256) -> Option<u64> {
		self.client.block_header(BlockId::Hash(hash)).map(|h| h.number())
	}

	fn is_major_importing(&self) -> bool {
		(self.sync_status)()
	}
}

impl<C: 'static> Broadcast for Mutex<IoChannel<ClientIoMessage<C>>> {
	fn take_at(&self, num: Option<u64>) {
		let num = match num {
			Some(n) => n,
			None => return,
		};

		trace!(target: "snapshot_watcher", "Snapshot requested at block #{}", num);

		if let Err(e) = self.lock().send(ClientIoMessage::TakeSnapshot(num)) {
			warn!("Snapshot watcher disconnected from IoService: {}", e);
		}
	}
}

/// A `ChainNotify` implementation which will trigger a snapshot event
/// at certain block numbers.
pub struct Watcher {
	oracle: Box<dyn Oracle>,
	broadcast: Box<dyn Broadcast>,
	period: u64,
	history: u64,
}

impl Watcher {
	/// Create a new `Watcher` which will trigger a snapshot event
	/// once every `period` blocks, but only after that block is
	/// `history` blocks old.
	pub fn new<F, C>(
		client: Arc<dyn BlockInfo>,
		sync_status: F,
		channel: IoChannel<ClientIoMessage<C>>,
		period: u64,
		history: u64
	) -> Self
		where
			F: 'static + Send + Sync + Fn() -> bool,
			C: 'static + Send + Sync,
	{
		Watcher {
			oracle: Box::new(StandardOracle { client, sync_status }),
			broadcast: Box::new(Mutex::new(channel)),
			period,
			history,
		}
	}

	#[cfg(any(test, feature = "test-helpers"))]
	/// Instantiate a `Watcher` using anything that impls `Oracle` and `Broadcast`. Test only.
	pub fn new_test(oracle: Box<dyn Oracle>, broadcast: Box<dyn Broadcast>, period: u64, history: u64) -> Self {
		Watcher { oracle, broadcast, period, history }
	}
}

impl ChainNotify for Watcher {
	fn new_blocks(&self, new_blocks: NewBlocks) {
		if self.oracle.is_major_importing() || new_blocks.has_more_blocks_to_import { return }

		trace!(target: "snapshot_watcher", "{} imported", new_blocks.imported.len());

		let highest = new_blocks.imported.into_iter()
			.filter_map(|h| self.oracle.to_number(h))
			.filter(|&num| num >= self.period + self.history)
			.map(|num| num - self.history)
			.filter(|num| num % self.period == 0)
			.fold(0, ::std::cmp::max);

		match highest {
			0 => self.broadcast.take_at(None),
			_ => self.broadcast.take_at(Some(highest)),
		}
	}
}
