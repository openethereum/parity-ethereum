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

//! Watcher for snapshot-related chain events.

use util::Mutex;
use client::{BlockChainClient, Client, ChainNotify};
use ids::BlockId;
use service::ClientIoMessage;

use io::IoChannel;
use util::{H256, Bytes};

use std::sync::Arc;

// helper trait for transforming hashes to numbers and checking if syncing.
trait Oracle: Send + Sync {
	fn to_number(&self, hash: H256) -> Option<u64>;

	fn is_major_importing(&self) -> bool;
}

struct StandardOracle<F> where F: 'static + Send + Sync + Fn() -> bool {
	client: Arc<Client>,
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

// helper trait for broadcasting a block to take a snapshot at.
trait Broadcast: Send + Sync {
	// take a snapshot optimistically at this block.
	fn take_at(&self, num: Option<u64>);

	// attempt to solidify snapshot now that confirmations have rolled in.
	fn solidify(&self, num: Option<u64>);
}

impl Broadcast for Mutex<IoChannel<ClientIoMessage>> {
	fn take_at(&self, num: Option<u64>) {
		let num = match num {
			Some(n) => n,
			None => return,
		};

		trace!(target: "snapshot_watcher", "broadcast: {}", num);

		if let Err(e) = self.lock().send(ClientIoMessage::TakeSnapshot(num)) {
			warn!("Snapshot watcher disconnected from IoService: {}", e);
		}
	}

	fn solidify(&self, num: Option<u64>) {
		let num = match num {
			Some(n) => n,
			None => return,
		};

		trace!(target: "snapshot_watcher", "broadcast: {}", num);

		if let Err(e) = self.lock().send(ClientIoMessage::SolidifySnapshot(num)) {
			warn!("Snapshot watcher disconnected from IoService: {}", e);
		}
	}
}

/// A `ChainNotify` implementation which will trigger a snapshot event
/// at certain block numbers.
pub struct Watcher {
	oracle: Box<Oracle>,
	broadcast: Box<Broadcast>,
	period: u64,
	create_delay: u64,
	propagate_delay: u64,
}

impl Watcher {
	/// Create a new `Watcher` which will trigger a snapshot event
	/// once every `period` blocks, but only after that block is
	/// `create_delay` blocks old.
	///
	/// The created snapshot will be "solidified" once that block
	/// is "propagate_delay" blocks old.
	///
	/// `create_delay` and `propagate_delay` should be sufficiently far
	/// enough apart so that creation finishes well before propagation.
	pub fn new<F: 'static + Send + Sync + Fn() -> bool>(
		client: Arc<Client>,
		sync_status: F,
		channel: IoChannel<ClientIoMessage>,
		period: u64,
		create_delay: u64,
		propagate_delay: u64
	) -> Self {
		Watcher {
			oracle: Box::new(StandardOracle {
				client: client,
				sync_status: sync_status,
			}),
			broadcast: Box::new(Mutex::new(channel)),
			period: period,
			create_delay: create_delay,
			propagate_delay: propagate_delay,
		}
	}
}

impl ChainNotify for Watcher {
	fn new_blocks(
		&self,
		imported: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_: Vec<Bytes>,
		_duration: u64)
	{
		if self.oracle.is_major_importing() { return }

		trace!(target: "snapshot_watcher", "{} imported", imported.len());

		let numbers: Vec<_> = imported.into_iter()
			.filter_map(|h| self.oracle.to_number(h))
			.collect();

		let find_highest = |delay| {
			numbers.iter()
			    .filter(|&&num| num >= self.period + delay)
				.map(|num| num - delay)
				.filter(|num| num % self.period == 0)
				.fold(0, ::std::cmp::max)
		};

		match find_highest(self.create_delay) {
			0 => self.broadcast.take_at(None),
			take_at => self.broadcast.take_at(Some(take_at)),
		}

		match find_highest(self.propagate_delay) {
			0 => self.broadcast.solidify(None),
			solidify_at => self.broadcast.solidify(Some(solidify_at)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{Broadcast, Oracle, Watcher};

	use client::ChainNotify;

	use util::{H256, U256};

	use std::collections::HashMap;

	struct TestOracle(HashMap<H256, u64>);

	impl Oracle for TestOracle {
		fn to_number(&self, hash: H256) -> Option<u64> {
			self.0.get(&hash).cloned()
		}

		fn is_major_importing(&self) -> bool { false }
	}

	struct TestBroadcast(Option<u64>, Option<u64>);
	impl Broadcast for TestBroadcast {
		fn take_at(&self, num: Option<u64>) {
			if num != self.0 {
				panic!("take_at wrong number. Expected {:?}, found {:?}", self.0, num);
			}
		}

		fn solidify(&self, num: Option<u64>) {
			if num != self.1 {
				panic!("solidify wrong number. Expected {:?}, found {:?}", self.1, num);
			}
		}
	}

	// helper harness for tests which expect a notification.
	fn harness(numbers: Vec<u64>, period: u64, create_delay: u64, propagate_delay: u64, broadcast: TestBroadcast) {
		let hashes: Vec<_> = numbers.clone().into_iter().map(|x| H256::from(U256::from(x))).collect();
		let map = hashes.clone().into_iter().zip(numbers).collect();

		let watcher = Watcher {
			oracle: Box::new(TestOracle(map)),
			broadcast: Box::new(broadcast),
			period: period,
			create_delay: create_delay,
			propagate_delay: propagate_delay,
		};

		watcher.new_blocks(
			hashes,
			vec![],
			vec![],
			vec![],
			vec![],
			vec![],
			0,
		);
	}

	// helper

	#[test]
	fn should_not_fire() {
		harness(vec![0], 5, 0, 0, TestBroadcast(None, None));
	}

	#[test]
	fn fires_once_for_two() {
		harness(vec![14, 15], 10, 5, 10, TestBroadcast(Some(10), None));
		harness(vec![25, 35], 10, 7, 5, TestBroadcast(None, Some(30)));
	}

	#[test]
	fn finds_highest() {
		harness(vec![15, 25], 10, 5, 7, TestBroadcast(Some(20), None));
		harness(vec![15, 25], 10, 3, 5, TestBroadcast(None, Some(20)));
	}

	#[test]
	fn doesnt_fire_before_history() {
		harness(vec![10, 11], 10, 5, 5, TestBroadcast(None, None));
	}
}
