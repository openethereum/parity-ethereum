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

use std::sync::Arc;
use ethcore::client::{TestBlockChainClient, BlockChainClient, BlockId, EachBlockWith, ChainInfo, BlockInfo};
use chain::{SyncState};
use super::helpers::*;
use {SyncConfig, WarpSync};

#[test]
fn two_peers() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(3);
	net.peer(1).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.peer(2).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.sync();
	assert!(net.peer(0).chain.block(BlockId::Number(1000)).is_some());
	assert_eq!(*net.peer(0).chain.blocks.read(), *net.peer(1).chain.blocks.read());
}

#[test]
fn long_chain() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(2);
	net.peer(1).chain.add_blocks(50000, EachBlockWith::Nothing);
	net.sync();
	assert!(net.peer(0).chain.block(BlockId::Number(50000)).is_some());
	assert_eq!(*net.peer(0).chain.blocks.read(), *net.peer(1).chain.blocks.read());
}

#[test]
fn status_after_sync() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(3);
	net.peer(1).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.peer(2).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.sync();
	let status = net.peer(0).sync.read().status();
	assert_eq!(status.state, SyncState::Idle);
}

#[test]
fn takes_few_steps() {
	let mut net = TestNet::new(3);
	net.peer(1).chain.add_blocks(100, EachBlockWith::Uncle);
	net.peer(2).chain.add_blocks(100, EachBlockWith::Uncle);
	let total_steps = net.sync();
	assert!(total_steps < 20);
}

#[test]
fn empty_blocks() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(3);
	for n in 0..200 {
		let with = if n % 2 == 0 { EachBlockWith::Nothing } else { EachBlockWith::Uncle };
		net.peer(1).chain.add_blocks(5, with.clone());
		net.peer(2).chain.add_blocks(5, with);
	}
	net.sync();
	assert!(net.peer(0).chain.block(BlockId::Number(1000)).is_some());
	assert_eq!(*net.peer(0).chain.blocks.read(), *net.peer(1).chain.blocks.read());
}

#[test]
fn forked() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(3);
	net.peer(0).chain.add_blocks(30, EachBlockWith::Uncle);
	net.peer(1).chain.add_blocks(30, EachBlockWith::Uncle);
	net.peer(2).chain.add_blocks(30, EachBlockWith::Uncle);
	net.peer(0).chain.add_blocks(10, EachBlockWith::Nothing); //fork
	net.peer(1).chain.add_blocks(20, EachBlockWith::Uncle);
	net.peer(2).chain.add_blocks(20, EachBlockWith::Uncle);
	net.peer(1).chain.add_blocks(10, EachBlockWith::Uncle); //fork between 1 and 2
	net.peer(2).chain.add_blocks(1, EachBlockWith::Nothing);
	// peer 1 has the best chain of 601 blocks
	let peer1_chain = net.peer(1).chain.numbers.read().clone();
	net.sync();
	assert_eq!(*net.peer(0).chain.difficulty.read(), *net.peer(1).chain.difficulty.read());
	assert_eq!(&*net.peer(0).chain.numbers.read(), &peer1_chain);
	assert_eq!(&*net.peer(1).chain.numbers.read(), &peer1_chain);
	assert_eq!(&*net.peer(2).chain.numbers.read(), &peer1_chain);
}

#[test]
fn forked_with_misbehaving_peer() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(3);

	let mut alt_spec = ::ethcore::spec::Spec::new_test();
	alt_spec.extra_data = b"fork".to_vec();
	// peer 0 is on a totally different chain with higher total difficulty
	net.peer_mut(0).chain = Arc::new(TestBlockChainClient::new_with_spec(alt_spec));
	net.peer(0).chain.add_blocks(50, EachBlockWith::Nothing);
	net.peer(1).chain.add_blocks(10, EachBlockWith::Nothing);
	net.peer(2).chain.add_blocks(10, EachBlockWith::Nothing);

	net.peer(1).chain.add_blocks(10, EachBlockWith::Nothing);
	net.peer(2).chain.add_blocks(20, EachBlockWith::Uncle);
	// peer 1 should sync to peer 2, others should not change
	let peer0_chain = net.peer(0).chain.numbers.read().clone();
	let peer2_chain = net.peer(2).chain.numbers.read().clone();
	net.sync();
	assert_eq!(&*net.peer(0).chain.numbers.read(), &peer0_chain);
	assert_eq!(&*net.peer(1).chain.numbers.read(), &peer2_chain);
	assert_eq!(&*net.peer(2).chain.numbers.read(), &peer2_chain);
}

#[test]
fn net_hard_fork() {
	::env_logger::try_init().ok();
	let ref_client = TestBlockChainClient::new();
	ref_client.add_blocks(50, EachBlockWith::Uncle);
	{
		let mut net = TestNet::new_with_fork(2, Some((50, ref_client.block_hash(BlockId::Number(50)).unwrap())));
		net.peer(0).chain.add_blocks(100, EachBlockWith::Uncle);
		net.sync();
		assert_eq!(net.peer(1).chain.chain_info().best_block_number, 100);
	}
	{
		let mut net = TestNet::new_with_fork(2, Some((50, ref_client.block_hash(BlockId::Number(50)).unwrap())));
		net.peer(0).chain.add_blocks(100, EachBlockWith::Nothing);
		net.sync();
		assert_eq!(net.peer(1).chain.chain_info().best_block_number, 0);
	}
}

#[test]
fn restart() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(3);
	net.peer(1).chain.add_blocks(1000, EachBlockWith::Uncle);
	net.peer(2).chain.add_blocks(1000, EachBlockWith::Uncle);

	net.sync();

	// make sure that sync has actually happened
	assert!(net.peer(0).chain.chain_info().best_block_number > 100);
	net.restart_peer(0);

	let status = net.peer(0).sync.read().status();
	assert_eq!(status.state, SyncState::Idle);
}

#[test]
fn status_empty() {
	let net = TestNet::new(2);
	assert_eq!(net.peer(0).sync.read().status().state, SyncState::Idle);
	let mut config = SyncConfig::default();
	config.warp_sync = WarpSync::Enabled;
	let net = TestNet::new_with_config(2, config);
	assert_eq!(net.peer(0).sync.read().status().state, SyncState::WaitingPeers);
}

#[test]
fn status_packet() {
	let mut net = TestNet::new(2);
	net.peer(0).chain.add_blocks(100, EachBlockWith::Uncle);
	net.peer(1).chain.add_blocks(1, EachBlockWith::Uncle);

	net.start();

	net.sync_step_peer(0);

	assert_eq!(1, net.peer(0).queue.read().len());
	assert_eq!(0x00, net.peer(0).queue.read()[0].packet_id);
}

#[test]
fn propagate_hashes() {
	let mut net = TestNet::new(6);
	net.peer(1).chain.add_blocks(10, EachBlockWith::Uncle);
	net.sync();

	net.peer(0).chain.add_blocks(10, EachBlockWith::Uncle);
	net.sync();
	net.trigger_chain_new_blocks(0); //first event just sets the marker
	net.trigger_chain_new_blocks(0);

	// 5 peers with NewHahses, 4 with blocks
	assert_eq!(9, net.peer(0).queue.read().len());
	let mut hashes = 0;
	let mut blocks = 0;
	for i in 0..net.peer(0).queue.read().len() {
		if net.peer(0).queue.read()[i].packet_id == 0x1 {
			hashes += 1;
		}
		if net.peer(0).queue.read()[i].packet_id == 0x7 {
			blocks += 1;
		}
	}
	assert_eq!(blocks, 4);
	assert_eq!(hashes, 5);
}

#[test]
fn propagate_blocks() {
	let mut net = TestNet::new(20);
	net.peer(1).chain.add_blocks(10, EachBlockWith::Uncle);
	net.sync();

	net.peer(0).chain.add_blocks(10, EachBlockWith::Uncle);
	net.trigger_chain_new_blocks(0); //first event just sets the marker
	net.trigger_chain_new_blocks(0);

	assert!(!net.peer(0).queue.read().is_empty());
	// NEW_BLOCK_PACKET
	let blocks = net.peer(0).queue.read().iter().filter(|p| p.packet_id == 0x7).count();
	assert!(blocks > 0);
}

#[test]
fn restart_on_malformed_block() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(2);
	net.peer(1).chain.add_blocks(5, EachBlockWith::Nothing);
	net.peer(1).chain.add_block(EachBlockWith::Nothing, |mut header| {
		header.set_extra_data(b"This extra data is way too long to be considered valid".to_vec());
		header
	});
	net.sync_steps(20);

	// This gets accepted just fine since the TestBlockChainClient performs no validation.
	// Probably remove this test?
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 6);
}

#[test]
fn reject_on_broken_chain() {
	let mut net = TestNet::new(2);
	net.peer(1).chain.add_blocks(10, EachBlockWith::Nothing);
	net.peer(1).chain.corrupt_block_parent(6);
	net.sync_steps(20);

	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 0);
}

#[test]
fn disconnect_on_unrelated_chain() {
	::env_logger::try_init().ok();
	let mut net = TestNet::new(2);
	net.peer(0).chain.set_history(Some(20));
	net.peer(1).chain.set_history(Some(20));
	net.restart_peer(0);
	net.restart_peer(1);
	net.peer(0).chain.add_blocks(500, EachBlockWith::Uncle);
	net.peer(1).chain.add_blocks(300, EachBlockWith::Nothing);
	net.sync();
	assert_eq!(net.disconnect_events, vec![(0, 0)]);
}
