// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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
	fn request_snapshot_at(&self, num: u64) {
		if let Err(e) = self.lock().send(ClientIoMessage::TakeSnapshot(num)) {
			warn!(target: "snapshot_watcher", "Snapshot watcher disconnected from IoService: {}", e);
		} else {
			trace!(target: "snapshot_watcher", "Snapshot requested at block #{}", num);
		}
	}
}

/// A `ChainNotify` implementation which will trigger a snapshot event
/// at certain block numbers.
pub struct Watcher {
	oracle: Box<dyn Oracle>,
	broadcast: Box<dyn Broadcast>,
	// How often we attempt to take a snapshot: only snapshot on blocknumbers that are multiples of
	// `period`. Always set to `SNAPSHOT_PERIOD`, i.e. 5000.
	period: u64,
	// Start snapshots `history` blocks from the tip. Always set to `SNAPSHOT_HISTORY`, i.e. 100.
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

		// Decide if it's time for a snapshot: the highest of the imported blocks is a multiple of 5000?
		let highest = new_blocks.imported.into_iter()
			// Convert block hashes to block numbers for all newly imported blocks
			.filter_map(|h| self.oracle.to_number(h))
			// Subtract `history` (i.e. `SNAPSHOT_HISTORY`, i.e. 100) from the block numbers to stay
			// clear of reorgs.
			.map(|num| num.saturating_sub(self.history) )
			// …filter out blocks that do not fall on the a multiple of `period`. This regulates the
			// frequency of snapshots and ensures more snapshots are produced from similar points in
			// the chain.
			.filter(|num| num % self.period == 0 )
			// Pick newest of the candidates: this is where we want to snapshot from.
			.fold(0, ::std::cmp::max);

		if highest > 0 {
			self.broadcast.request_snapshot_at(highest);
		}
	}
}
