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
use ethcore::client::{BlockChainClient, BlockID, EachBlockWith};
use chain::{SyncState};
use super::helpers::*;

#[test]
fn two_peers() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.peer_mut(2).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.sync();
	assert!(net.peer(0).chain.block(BlockID::Number(1000)).is_some());
	assert_eq!(net.peer(0).chain.blocks.read().unwrap().deref(), net.peer(1).chain.blocks.read().unwrap().deref());
}

#[test]
fn long_chain() {
	::env_logger::init().ok();
	let mut net = TestNet::new(2);
	net.peer_mut(1).chain.add_blocks(50000, EachBlockWith::Nothing);
	net.sync();
	assert!(net.peer(0).chain.block(BlockID::Number(50000)).is_some());
	assert_eq!(net.peer(0).chain.blocks.read().unwrap().deref(), net.peer(1).chain.blocks.read().unwrap().deref());
}

#[test]
fn status_after_sync() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.peer_mut(2).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.sync();
	let status = net.peer(0).sync.status();
	assert_eq!(status.state, SyncState::Idle);
}

#[test]
fn takes_few_steps() {
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(100, EachBlockWith::Uncle);
	net.peer_mut(2).chain.add_blocks(100, EachBlockWith::Uncle);
	let total_steps = net.sync();
	assert!(total_steps < 20);
}

#[test]
fn empty_blocks() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	for n in 0..200 {
		let with = if n % 2 == 0 { EachBlockWith::Nothing } else { EachBlockWith::Uncle };
		net.peer_mut(1).chain.add_blocks(5, with.clone());
		net.peer_mut(2).chain.add_blocks(5, with);
	}
	net.sync();
	assert!(net.peer(0).chain.block(BlockID::Number(1000)).is_some());
	assert_eq!(net.peer(0).chain.blocks.read().unwrap().deref(), net.peer(1).chain.blocks.read().unwrap().deref());
}

#[test]
fn forked() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(0).chain.add_blocks(300, EachBlockWith::Uncle);
	net.peer_mut(1).chain.add_blocks(300, EachBlockWith::Uncle);
	net.peer_mut(2).chain.add_blocks(300, EachBlockWith::Uncle);
	net.peer_mut(0).chain.add_blocks(100, EachBlockWith::Nothing); //fork
	net.peer_mut(1).chain.add_blocks(200, EachBlockWith::Uncle);
	net.peer_mut(2).chain.add_blocks(200, EachBlockWith::Uncle);
	net.peer_mut(1).chain.add_blocks(100, EachBlockWith::Uncle); //fork between 1 and 2
	net.peer_mut(2).chain.add_blocks(10, EachBlockWith::Nothing);
	// peer 1 has the best chain of 601 blocks
	let peer1_chain = net.peer(1).chain.numbers.read().unwrap().clone();
	net.sync();
	assert_eq!(net.peer(0).chain.difficulty.read().unwrap().deref(), net.peer(1).chain.difficulty.read().unwrap().deref());
	assert_eq!(net.peer(0).chain.numbers.read().unwrap().deref(), &peer1_chain);
	assert_eq!(net.peer(1).chain.numbers.read().unwrap().deref(), &peer1_chain);
	assert_eq!(net.peer(2).chain.numbers.read().unwrap().deref(), &peer1_chain);
}

#[test]
fn restart() {
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.peer_mut(2).chain.add_blocks(1000, EachBlockWith::Uncle);

	net.sync_steps(8);

	// make sure that sync has actually happened
	assert!(net.peer(0).chain.chain_info().best_block_number > 100);
	net.restart_peer(0);

	let status = net.peer(0).sync.status();
	assert_eq!(status.state, SyncState::ChainHead);
}

#[test]
fn status_empty() {
	let net = TestNet::new(2);
	assert_eq!(net.peer(0).sync.status().state, SyncState::Idle);
}

#[test]
fn status_packet() {
	let mut net = TestNet::new(2);
	net.peer_mut(0).chain.add_blocks(100, EachBlockWith::Uncle);
	net.peer_mut(1).chain.add_blocks(1, EachBlockWith::Uncle);

	net.start();

	net.sync_step_peer(0);

	assert_eq!(1, net.peer(0).queue.len());
	assert_eq!(0x00, net.peer(0).queue[0].packet_id);
}

#[test]
fn propagate_hashes() {
	let mut net = TestNet::new(6);
	net.peer_mut(1).chain.add_blocks(10, EachBlockWith::Uncle);
	net.sync();

	net.peer_mut(0).chain.add_blocks(10, EachBlockWith::Uncle);
	net.sync();
	net.trigger_chain_new_blocks(0); //first event just sets the marker
	net.trigger_chain_new_blocks(0);

	// 5 peers to sync
	assert_eq!(5, net.peer(0).queue.len());
	let mut hashes = 0;
	let mut blocks = 0;
	for i in 0..5 {
		if net.peer(0).queue[i].packet_id == 0x1 {
			hashes += 1;
		}
		if net.peer(0).queue[i].packet_id == 0x7 {
			blocks += 1;
		}
	}
	assert!(blocks + hashes == 5);
}

#[test]
fn propagate_blocks() {
	let mut net = TestNet::new(2);
	net.peer_mut(1).chain.add_blocks(10, EachBlockWith::Uncle);
	net.sync();

	net.peer_mut(0).chain.add_blocks(10, EachBlockWith::Uncle);
	net.trigger_chain_new_blocks(0); //first event just sets the marker
	net.trigger_chain_new_blocks(0);

	assert!(!net.peer(0).queue.is_empty());
	// NEW_BLOCK_PACKET
	assert_eq!(0x07, net.peer(0).queue[0].packet_id);
}

#[test]
fn restart_on_malformed_block() {
	let mut net = TestNet::new(2);
	net.peer_mut(1).chain.add_blocks(10, EachBlockWith::Uncle);
	net.peer_mut(1).chain.corrupt_block(6);
	net.sync_steps(20);

	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 5);
}

#[test]
fn restart_on_broken_chain() {
	let mut net = TestNet::new(2);
	net.peer_mut(1).chain.add_blocks(10, EachBlockWith::Uncle);
	net.peer_mut(1).chain.corrupt_block_parent(6);
	net.sync_steps(20);

	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 5);
}
