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

use parking_lot::Mutex;
use client::{BlockChainClient, Client, ChainNotify};
use ids::BlockId;
use service::ClientIoMessage;

use io::IoChannel;
use bigint::hash::H256;
use util::Bytes;

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
	fn take_at(&self, num: Option<u64>);
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
}

/// A `ChainNotify` implementation which will trigger a snapshot event
/// at certain block numbers.
pub struct Watcher {
	oracle: Box<Oracle>,
	broadcast: Box<Broadcast>,
	period: u64,
	history: u64,
}

impl Watcher {
	/// Create a new `Watcher` which will trigger a snapshot event
	/// once every `period` blocks, but only after that block is
	/// `history` blocks old.
	pub fn new<F>(client: Arc<Client>, sync_status: F, channel: IoChannel<ClientIoMessage>, period: u64, history: u64) -> Self
		where F: 'static + Send + Sync + Fn() -> bool
	{
		Watcher {
			oracle: Box::new(StandardOracle {
				client: client,
				sync_status: sync_status,
			}),
			broadcast: Box::new(Mutex::new(channel)),
			period: period,
			history: history,
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

		let highest = imported.into_iter()
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

#[cfg(test)]
mod tests {
	use super::{Broadcast, Oracle, Watcher};

	use client::ChainNotify;

	use bigint::prelude::U256;
	use bigint::hash::H256;

	use std::collections::HashMap;

	struct TestOracle(HashMap<H256, u64>);

	impl Oracle for TestOracle {
		fn to_number(&self, hash: H256) -> Option<u64> {
			self.0.get(&hash).cloned()
		}

		fn is_major_importing(&self) -> bool { false }
	}

	struct TestBroadcast(Option<u64>);
	impl Broadcast for TestBroadcast {
		fn take_at(&self, num: Option<u64>) {
			if num != self.0 {
				panic!("Watcher broadcast wrong number. Expected {:?}, found {:?}", self.0, num);
			}
		}
	}

	// helper harness for tests which expect a notification.
	fn harness(numbers: Vec<u64>, period: u64, history: u64, expected: Option<u64>) {
		let hashes: Vec<_> = numbers.clone().into_iter().map(|x| H256::from(U256::from(x))).collect();
		let map = hashes.clone().into_iter().zip(numbers).collect();

		let watcher = Watcher {
			oracle: Box::new(TestOracle(map)),
			broadcast: Box::new(TestBroadcast(expected)),
			period: period,
			history: history,
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
		harness(vec![0], 5, 0, None);
	}

	#[test]
	fn fires_once_for_two() {
		harness(vec![14, 15], 10, 5, Some(10));
	}

	#[test]
	fn finds_highest() {
		harness(vec![15, 25], 10, 5, Some(20));
	}

	#[test]
	fn doesnt_fire_before_history() {
		harness(vec![10, 11], 10, 5, None);
	}
}
