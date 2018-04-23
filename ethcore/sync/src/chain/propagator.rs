// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use bytes::Bytes;
use ethereum_types::{H256};
use ethcore::client::{BlockChainInfo};
use ethcore::header::{BlockNumber};
use network::{PeerId, PacketId};
use rand::Rng;
use rlp::{Encodable, RlpStream};
use sync_io::SyncIo;
use std::cmp;
use std::collections::{HashSet};
use transaction::SignedTransaction;

use super::{
	random,
	ChainSync,

	MAX_PEER_LAG_PROPAGATION,
	MAX_PEERS_PROPAGATION,
	MAX_TRANSACTION_PACKET_SIZE,
	MAX_TRANSACTIONS_TO_PROPAGATE,
	MIN_PEERS_PROPAGATION,

	CONSENSUS_DATA_PACKET,
	NEW_BLOCK_HASHES_PACKET,
	NEW_BLOCK_PACKET,
	PRIVATE_TRANSACTION_PACKET,
	SIGNED_PRIVATE_TRANSACTION_PACKET,
	TRANSACTIONS_PACKET,
};

/// Checks if peer is able to process service transactions
fn accepts_service_transaction(client_id: &str) -> bool {
	// Parity versions starting from this will accept service-transactions
	const SERVICE_TRANSACTIONS_VERSION: (u32, u32) = (1u32, 6u32);
	// Parity client string prefix
	const PARITY_CLIENT_ID_PREFIX: &'static str = "Parity/v";

	if !client_id.starts_with(PARITY_CLIENT_ID_PREFIX) {
		return false;
	}
	let ver: Vec<u32> = client_id[PARITY_CLIENT_ID_PREFIX.len()..].split('.')
		.take(2)
		.filter_map(|s| s.parse().ok())
		.collect();
	ver.len() == 2 && (ver[0] > SERVICE_TRANSACTIONS_VERSION.0 || (ver[0] == SERVICE_TRANSACTIONS_VERSION.0 && ver[1] >= SERVICE_TRANSACTIONS_VERSION.1))
}

pub struct SyncPropagator {}

impl SyncPropagator {
	/// propagates latest block to a set of peers
	pub fn propagate_blocks(sync: &mut ChainSync, chain_info: &BlockChainInfo, io: &mut SyncIo, blocks: &[H256], peers: &[PeerId]) -> usize {
		trace!(target: "sync", "Sending NewBlocks to {:?}", peers);
		let mut sent = 0;
		for peer_id in peers {
			if blocks.is_empty() {
				let rlp =  ChainSync::create_latest_block_rlp(io.chain());
				SyncPropagator::send_packet(io, *peer_id, NEW_BLOCK_PACKET, rlp);
			} else {
				for h in blocks {
					let rlp =  ChainSync::create_new_block_rlp(io.chain(), h);
					SyncPropagator::send_packet(io, *peer_id, NEW_BLOCK_PACKET, rlp);
				}
			}
			if let Some(ref mut peer) = sync.peers.get_mut(peer_id) {
				peer.latest_hash = chain_info.best_block_hash.clone();
			}
			sent += 1;
		}
		sent
	}

	/// propagates new known hashes to all peers
	pub fn propagate_new_hashes(sync: &mut ChainSync, chain_info: &BlockChainInfo, io: &mut SyncIo, peers: &[PeerId]) -> usize {
		trace!(target: "sync", "Sending NewHashes to {:?}", peers);
		let mut sent = 0;
		let last_parent = *io.chain().best_block_header().parent_hash();
		for peer_id in peers {
			sent += match ChainSync::create_new_hashes_rlp(io.chain(), &last_parent, &chain_info.best_block_hash) {
				Some(rlp) => {
					{
						if let Some(ref mut peer) = sync.peers.get_mut(peer_id) {
							peer.latest_hash = chain_info.best_block_hash.clone();
						}
					}
					SyncPropagator::send_packet(io, *peer_id, NEW_BLOCK_HASHES_PACKET, rlp);
					1
				},
				None => 0
			}
		}
		sent
	}

	/// propagates new transactions to all peers
	pub fn propagate_new_transactions(sync: &mut ChainSync, io: &mut SyncIo) -> usize {
		// Early out if nobody to send to.
		if sync.peers.is_empty() {
			return 0;
		}

		let transactions = io.chain().ready_transactions();
		if transactions.is_empty() {
			return 0;
		}

		let (transactions, service_transactions): (Vec<_>, Vec<_>) = transactions.iter()
			.map(|tx| tx.signed())
			.partition(|tx| !tx.gas_price.is_zero());

		// usual transactions could be propagated to all peers
		let mut affected_peers = HashSet::new();
		if !transactions.is_empty() {
			let peers = SyncPropagator::select_peers_for_transactions(sync, |_| true);
			affected_peers = SyncPropagator::propagate_transactions_to_peers(sync, io, peers, transactions);
		}

		// most of times service_transactions will be empty
		// => there's no need to merge packets
		if !service_transactions.is_empty() {
			let service_transactions_peers = SyncPropagator::select_peers_for_transactions(sync, |peer_id| accepts_service_transaction(&io.peer_info(*peer_id)));
			let service_transactions_affected_peers = SyncPropagator::propagate_transactions_to_peers(sync, io, service_transactions_peers, service_transactions);
			affected_peers.extend(&service_transactions_affected_peers);
		}

		affected_peers.len()
	}

	fn propagate_transactions_to_peers(sync: &mut ChainSync, io: &mut SyncIo, peers: Vec<PeerId>, transactions: Vec<&SignedTransaction>) -> HashSet<PeerId> {
		let all_transactions_hashes = transactions.iter()
			.map(|tx| tx.hash())
			.collect::<HashSet<H256>>();
		let all_transactions_rlp = {
			let mut packet = RlpStream::new_list(transactions.len());
			for tx in &transactions { packet.append(&**tx); }
			packet.out()
		};

		// Clear old transactions from stats
		sync.transactions_stats.retain(&all_transactions_hashes);

		// sqrt(x)/x scaled to max u32
		let block_number = io.chain().chain_info().best_block_number;

		let lucky_peers = {
			peers.into_iter()
				.filter_map(|peer_id| {
					let stats = &mut sync.transactions_stats;
					let peer_info = sync.peers.get_mut(&peer_id)
						.expect("peer_id is form peers; peers is result of select_peers_for_transactions; select_peers_for_transactions selects peers from self.peers; qed");

					// Send all transactions
					if peer_info.last_sent_transactions.is_empty() {
						// update stats
						for hash in &all_transactions_hashes {
							let id = io.peer_session_info(peer_id).and_then(|info| info.id);
							stats.propagated(hash, id, block_number);
						}
						peer_info.last_sent_transactions = all_transactions_hashes.clone();
						return Some((peer_id, all_transactions_hashes.len(), all_transactions_rlp.clone()));
					}

					// Get hashes of all transactions to send to this peer
					let to_send = all_transactions_hashes.difference(&peer_info.last_sent_transactions)
						.take(MAX_TRANSACTIONS_TO_PROPAGATE)
						.cloned()
						.collect::<HashSet<_>>();
					if to_send.is_empty() {
						return None;
					}

					// Construct RLP
					let (packet, to_send) = {
						let mut to_send = to_send;
						let mut packet = RlpStream::new();
						packet.begin_unbounded_list();
						let mut pushed = 0;
						for tx in &transactions {
							let hash = tx.hash();
							if to_send.contains(&hash) {
								let mut transaction = RlpStream::new();
								tx.rlp_append(&mut transaction);
								let appended = packet.append_raw_checked(&transaction.drain(), 1, MAX_TRANSACTION_PACKET_SIZE);
								if !appended {
									// Maximal packet size reached just proceed with sending
									debug!("Transaction packet size limit reached. Sending incomplete set of {}/{} transactions.", pushed, to_send.len());
									to_send = to_send.into_iter().take(pushed).collect();
									break;
								}
								pushed += 1;
							}
						}
						packet.complete_unbounded_list();
						(packet, to_send)
					};

					// Update stats
					let id = io.peer_session_info(peer_id).and_then(|info| info.id);
					for hash in &to_send {
						// update stats
						stats.propagated(hash, id, block_number);
					}

					peer_info.last_sent_transactions = all_transactions_hashes
						.intersection(&peer_info.last_sent_transactions)
						.chain(&to_send)
						.cloned()
						.collect();
					Some((peer_id, to_send.len(), packet.out()))
				})
				.collect::<Vec<_>>()
		};

		// Send RLPs
		let mut peers = HashSet::new();
		if lucky_peers.len() > 0 {
			let mut max_sent = 0;
			let lucky_peers_len = lucky_peers.len();
			for (peer_id, sent, rlp) in lucky_peers {
				peers.insert(peer_id);
				SyncPropagator::send_packet(io, peer_id, TRANSACTIONS_PACKET, rlp);
				trace!(target: "sync", "{:02} <- Transactions ({} entries)", peer_id, sent);
				max_sent = cmp::max(max_sent, sent);
			}
			debug!(target: "sync", "Sent up to {} transactions to {} peers.", max_sent, lucky_peers_len);
		}

		peers
	}

	pub fn propagate_latest_blocks(sync: &mut ChainSync, io: &mut SyncIo, sealed: &[H256]) {
		let chain_info = io.chain().chain_info();
		if (((chain_info.best_block_number as i64) - (sync.last_sent_block_number as i64)).abs() as BlockNumber) < MAX_PEER_LAG_PROPAGATION {
			let mut peers = sync.get_lagging_peers(&chain_info);
			if sealed.is_empty() {
				let hashes = SyncPropagator::propagate_new_hashes(sync, &chain_info, io, &peers);
				peers = ChainSync::select_random_peers(&peers);
				let blocks = SyncPropagator::propagate_blocks(sync, &chain_info, io, sealed, &peers);
				if blocks != 0 || hashes != 0 {
					trace!(target: "sync", "Sent latest {} blocks and {} hashes to peers.", blocks, hashes);
				}
			} else {
				SyncPropagator::propagate_blocks(sync, &chain_info, io, sealed, &peers);
				SyncPropagator::propagate_new_hashes(sync, &chain_info, io, &peers);
				trace!(target: "sync", "Sent sealed block to all peers");
			};
		}
		sync.last_sent_block_number = chain_info.best_block_number;
	}

	/// Distribute valid proposed blocks to subset of current peers.
	pub fn propagate_proposed_blocks(sync: &mut ChainSync, io: &mut SyncIo, proposed: &[Bytes]) {
		let peers = sync.get_consensus_peers();
		trace!(target: "sync", "Sending proposed blocks to {:?}", peers);
		for block in proposed {
			let rlp = ChainSync::create_block_rlp(
				block,
				io.chain().chain_info().total_difficulty
			);
			for peer_id in &peers {
				SyncPropagator::send_packet(io, *peer_id, NEW_BLOCK_PACKET, rlp.clone());
			}
		}
	}

	/// Broadcast consensus message to peers.
	pub fn propagate_consensus_packet(sync: &mut ChainSync, io: &mut SyncIo, packet: Bytes) {
		let lucky_peers = ChainSync::select_random_peers(&sync.get_consensus_peers());
		trace!(target: "sync", "Sending consensus packet to {:?}", lucky_peers);
		for peer_id in lucky_peers {
			SyncPropagator::send_packet(io, peer_id, CONSENSUS_DATA_PACKET, packet.clone());
		}
	}

	/// Broadcast private transaction message to peers.
	pub fn propagate_private_transaction(sync: &mut ChainSync, io: &mut SyncIo, packet: Bytes) {
		let lucky_peers = ChainSync::select_random_peers(&sync.get_private_transaction_peers());
		trace!(target: "sync", "Sending private transaction packet to {:?}", lucky_peers);
		for peer_id in lucky_peers {
			SyncPropagator::send_packet(io, peer_id, PRIVATE_TRANSACTION_PACKET, packet.clone());
		}
	}

	/// Broadcast signed private transaction message to peers.
	pub fn propagate_signed_private_transaction(sync: &mut ChainSync, io: &mut SyncIo, packet: Bytes) {
		let lucky_peers = ChainSync::select_random_peers(&sync.get_private_transaction_peers());
		trace!(target: "sync", "Sending signed private transaction packet to {:?}", lucky_peers);
		for peer_id in lucky_peers {
			SyncPropagator::send_packet(io, peer_id, SIGNED_PRIVATE_TRANSACTION_PACKET, packet.clone());
		}
	}

	fn select_peers_for_transactions<F>(sync: &ChainSync, filter: F) -> Vec<PeerId>
		where F: Fn(&PeerId) -> bool {
		// sqrt(x)/x scaled to max u32
		let fraction = ((sync.peers.len() as f64).powf(-0.5) * (u32::max_value() as f64).round()) as u32;
		let small = sync.peers.len() < MIN_PEERS_PROPAGATION;

		let mut random = random::new();
		sync.peers.keys()
			.cloned()
			.filter(filter)
			.filter(|_| small || random.next_u32() < fraction)
			.take(MAX_PEERS_PROPAGATION)
			.collect()
	}

	/// Generic packet sender
	fn send_packet(sync: &mut SyncIo, peer_id: PeerId, packet_id: PacketId, packet: Bytes) {
		if let Err(e) = sync.send(peer_id, packet_id, packet) {
			debug!(target:"sync", "Error sending packet: {:?}", e);
			sync.disconnect_peer(peer_id);
		}
	}
}

#[cfg(test)]
mod tests {

}
