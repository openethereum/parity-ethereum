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
use ethcore::client::{TestBlockChainClient, BlockChainClient};
use io::SyncIo;
use chain::ChainSync;
use ethminer::Miner;
use ::SyncConfig;

pub struct TestIo<'p> {
	pub chain: &'p mut TestBlockChainClient,
	pub queue: &'p mut VecDeque<TestPacket>,
	pub sender: Option<PeerId>,
}

impl<'p> TestIo<'p> {
	pub fn new(chain: &'p mut TestBlockChainClient, queue: &'p mut VecDeque<TestPacket>, sender: Option<PeerId>) -> TestIo<'p> {
		TestIo {
			chain: chain,
			queue: queue,
			sender: sender
		}
	}
}

impl<'p> SyncIo for TestIo<'p> {
	fn disable_peer(&mut self, _peer_id: PeerId) {
	}

	fn disconnect_peer(&mut self, _peer_id: PeerId) {
	}

	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		self.queue.push_back(TestPacket {
			data: data,
			packet_id: packet_id,
			recipient: self.sender.unwrap()
		});
		Ok(())
	}

	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError> {
		self.queue.push_back(TestPacket {
			data: data,
			packet_id: packet_id,
			recipient: peer_id,
		});
		Ok(())
	}

	fn chain(&self) -> &BlockChainClient {
		self.chain
	}
}

pub struct TestPacket {
	pub data: Bytes,
	pub packet_id: PacketId,
	pub recipient: PeerId,
}

pub struct TestPeer {
	pub chain: TestBlockChainClient,
	pub sync: ChainSync,
	pub queue: VecDeque<TestPacket>,
}

pub struct TestNet {
	pub peers: Vec<TestPeer>,
	pub started: bool,
}

impl TestNet {
	pub fn new(n: usize) -> TestNet {
		let mut net = TestNet {
			peers: Vec::new(),
			started: false,
		};
		for _ in 0..n {
			net.peers.push(TestPeer {
				chain: TestBlockChainClient::new(),
				sync: ChainSync::new(SyncConfig::default(), Miner::new(false)),
				queue: VecDeque::new(),
			});
		}
		net
	}

	pub fn peer(&self, i: usize) -> &TestPeer {
		self.peers.get(i).unwrap()
	}

	pub fn peer_mut(&mut self, i: usize) -> &mut TestPeer {
		self.peers.get_mut(i).unwrap()
	}

	pub fn start(&mut self) {
		for peer in 0..self.peers.len() {
			for client in 0..self.peers.len() {
				if peer != client {
					let mut p = self.peers.get_mut(peer).unwrap();
					p.sync.on_peer_connected(&mut TestIo::new(&mut p.chain, &mut p.queue, Some(client as PeerId)), client as PeerId);
				}
			}
		}
	}

	pub fn sync_step(&mut self) {
		for peer in 0..self.peers.len() {
			if let Some(packet) = self.peers[peer].queue.pop_front() {
				let mut p = self.peers.get_mut(packet.recipient).unwrap();
				trace!("--- {} -> {} ---", peer, packet.recipient);
				p.sync.on_packet(&mut TestIo::new(&mut p.chain, &mut p.queue, Some(peer as PeerId)), peer as PeerId, packet.packet_id, &packet.data);
				trace!("----------------");
			}
			let mut p = self.peers.get_mut(peer).unwrap();
			p.sync.maintain_sync(&mut TestIo::new(&mut p.chain, &mut p.queue, None));
		}
	}

	pub fn sync_step_peer(&mut self, peer_num: usize) {
		let mut peer = self.peer_mut(peer_num);
		peer.sync.maintain_sync(&mut TestIo::new(&mut peer.chain, &mut peer.queue, None));
	}

	pub fn restart_peer(&mut self, i: usize) {
		let peer = self.peer_mut(i);
		peer.sync.restart(&mut TestIo::new(&mut peer.chain, &mut peer.queue, None));
	}

	pub fn sync(&mut self) -> u32 {
		self.start();
		let mut total_steps = 0;
		while !self.done() {
			self.sync_step();
			total_steps = total_steps + 1;
		}
		total_steps
	}

	pub fn sync_steps(&mut self, count: usize) {
		if !self.started {
			self.start();
			self.started = true;
		}
		for _ in 0..count {
			self.sync_step();
		}
	}

	pub fn done(&self) -> bool {
		self.peers.iter().all(|p| p.queue.is_empty())
	}

	pub fn trigger_chain_new_blocks(&mut self, peer_id: usize) {
		let mut peer = self.peer_mut(peer_id);
		peer.sync.chain_new_blocks(&mut TestIo::new(&mut peer.chain, &mut peer.queue, None), &[], &[], &[], &[]);
	}
}
