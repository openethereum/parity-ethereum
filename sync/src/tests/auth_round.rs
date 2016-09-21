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

use util::*;
use ethcore::spec::Spec;
use super::mocknet::*;
use std::thread::sleep;
use std::time::Duration;
use ethcore::client::BlockChainClient;

fn authorities() -> Vec<H256> { vec!["1".sha3(), "2".sha3()] }

#[test]
fn three_peer_tx_seal() {
	::env_logger::init().ok();
	let mut net = MockNet::new_with_spec(2, authorities(), &Spec::new_test_round);
	net.peer(1).issue_rand_tx();
	sleep(Duration::from_secs(1));
	net.sync();
	sleep(Duration::from_secs(1));
	net.sync();
	println!("{:?}", net.peer(0).client.chain_info());
	println!("{:?}", net.peer(1).client.chain_info());
	net.is_synced(1);
}

#[test]
fn issue_many() {
	::env_logger::init().ok();
	let mut net = MockNet::new_with_spec(2, vec!["1".sha3()], &Spec::new_test_round);
	net.peer(1).issue_rand_txs(5);
	sleep(Duration::from_secs(1));
	net.sync();
	net.peer(0).issue_rand_txs(5);
	net.sync();
	println!("{:?}", net.peer(0).client.chain_info());
	println!("{:?}", net.peer(1).client.chain_info());
	net.is_synced(10);
}

#[test]
fn rand_simulation() {
	::env_logger::init().ok();
	let mut net = MockNet::new_with_spec(3, authorities(), &Spec::new_test_round);

	net.rand_simulation(10);

	println!("{:?}", net.peer(0).client.chain_info());
	println!("{:?}", net.peer(1).client.chain_info());
	net.is_synced(10);
}
