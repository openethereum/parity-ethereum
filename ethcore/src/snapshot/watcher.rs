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

//! Watcher for snapshot-related chain events.

use client::{BlockChainClient, Client, ChainNotify};
use ids::BlockID;
use service::ClientIoMessage;
use views::HeaderView;

use io::IoChannel;
use util::hash::H256;

use std::sync::Arc;

// helper trait for transforming hashes to numbers.
trait HashToNumber: Send + Sync {
	fn to_number(&self, hash: H256) -> Option<u64>;
}

impl HashToNumber for Client {
	fn to_number(&self, hash: H256) -> Option<u64> {
		self.block_header(BlockID::Hash(hash)).map(|h| HeaderView::new(&h).number())
	}
}

/// A `ChainNotify` implementation which will trigger a snapshot event
/// at certain block numbers.
pub struct Watcher {
	oracle: Arc<HashToNumber>,
	channel: IoChannel<ClientIoMessage>,
	period: u64,
	history: u64,
}

impl Watcher {
	/// Create a new `Watcher` which will trigger a snapshot event
	/// once every `period` blocks, but only after that block is
	/// `history` blocks old.
	pub fn new(client: Arc<Client>, channel: IoChannel<ClientIoMessage>, period: u64, history: u64) -> Self {
		Watcher {
			oracle: client,
			channel: channel,
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
		_duration: u64)
	{

		let highest = imported.into_iter()
			.filter_map(|h| self.oracle.to_number(h))
			.filter(|&num| num >= self.period + self.history)
			.map(|num| num - self.history)
			.filter(|num| num % self.period == 0)
			.fold(0, ::std::cmp::max);

		if highest != 0 {
			if let Err(e) = self.channel.send(ClientIoMessage::TakeSnapshot(highest)) {
				warn!("Snapshot watcher disconnected from IoService: {}", e);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{HashToNumber, Watcher};

	use client::ChainNotify;
	use service::ClientIoMessage;

	use util::{H256, U256, Mutex};
	use io::{IoContext, IoHandler, IoService};

	use std::collections::HashMap;
	use std::sync::Arc;

	struct TestOracle(HashMap<H256, u64>);

	impl HashToNumber for TestOracle {
		fn to_number(&self, hash: H256) -> Option<u64> {
			self.0.get(&hash).cloned()
		}
	}

	struct Handler(Arc<Mutex<Vec<u64>>>);

	impl IoHandler<ClientIoMessage> for Handler {
		fn message(&self, _context: &IoContext<ClientIoMessage>, message: &ClientIoMessage) {
			match *message {
				ClientIoMessage::TakeSnapshot(num) => self.0.lock().push(num),
				_ => {}
			}
		}
	}

	// helper harness for tests.
	fn harness(numbers: Vec<u64>, period: u64, history: u64) -> Vec<u64> {
		let events = Arc::new(Mutex::new(Vec::new()));

		let service = IoService::start().unwrap();
		service.register_handler(Arc::new(Handler(events.clone()))).unwrap();

		let hashes: Vec<_> = numbers.clone().into_iter().map(|x| H256::from(U256::from(x))).collect();
		let mut map = hashes.clone().into_iter().zip(numbers).collect();

		let watcher = Watcher {
			oracle: Arc::new(TestOracle(map)),
			channel: service.channel(),
			period: period,
			history: history,
		};

		watcher.new_blocks(
			hashes,
			vec![],
			vec![],
			vec![],
			vec![],
			0,
		);

		drop(service);

		// binding necessary for compilation.
		let v = events.lock().clone();
		v
	}

	#[test]
	fn should_not_fire() {
		let events = harness(vec![0], 5, 0);
		assert_eq!(events, vec![]);
	}

	#[test]
	fn fires_once_for_two() {
		let events = harness(vec![14, 15], 10, 5);
		assert_eq!(events, vec![10]);
	}

	#[test]
	fn finds_highest() {
		let events = harness(vec![15, 25], 10, 5);
		assert_eq!(events, vec![20]);
	}

	#[test]
	fn doesnt_fire_before_history() {
		let events = harness(vec![10, 11], 10, 5);
		assert_eq!(events, vec![]);
	}
}