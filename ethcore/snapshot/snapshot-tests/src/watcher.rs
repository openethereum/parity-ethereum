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

//! Tests for block RLP encoding

use std::collections::HashMap;
use std::time::Duration;

use client_traits::ChainNotify;
use common_types::chain_notify::{NewBlocks, ChainRoute};
use ethereum_types::{H256, U256, BigEndianHash};

use snapshot::{
	Broadcast,
	Oracle,
	test_helpers::Watcher,
};

struct TestOracle(HashMap<H256, u64>);

impl Oracle for TestOracle {
	fn to_number(&self, hash: H256) -> Option<u64> {
		self.0.get(&hash).cloned()
	}

	fn is_major_importing(&self) -> bool { false }
}

struct TestBroadcast(Option<u64>);
impl Broadcast for TestBroadcast {
	fn request_snapshot_at(&self, num: u64) {
		if Some(num) != self.0 {
			panic!("Watcher broadcast wrong number. Expected {:?}, found {:?}", self.0, num);
		}
	}
}

// helper harness for tests which expect a notification.
fn harness(numbers: Vec<u64>, period: u64, history: u64, expected: Option<u64>) {
	const DURATION_ZERO: Duration = Duration::from_millis(0);

	let hashes: Vec<_> = numbers.clone().into_iter().map(|x| BigEndianHash::from_uint(&U256::from(x))).collect();
	let map = hashes.clone().into_iter().zip(numbers).collect();
	let watcher = Watcher::new_test(
		Box::new(TestOracle(map)),
		Box::new(TestBroadcast(expected)),
		period,
		history,
	);

	watcher.new_blocks(NewBlocks::new(
		hashes,
		vec![],
		ChainRoute::default(),
		vec![],
		vec![],
		DURATION_ZERO,
		false
	));
}

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
