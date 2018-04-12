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

use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry;
use smallvec::SmallVec;
use hash::{keccak, KECCAK_NULL_RLP, KECCAK_EMPTY_LIST_RLP};
use heapsize::HeapSizeOf;
use ethereum_types::H256;
use triehash::ordered_trie_root;
use bytes::Bytes;
use rlp::{Rlp, RlpStream, DecoderError};
use network;
use ethcore::encoded::Block;
use ethcore::views::{HeaderView, BodyView};
use ethcore::header::Header as BlockHeader;

known_heap_size!(0, HeaderId);

type SmallHashVec = SmallVec<[H256; 1]>;

/// Block data with optional body.
struct SyncBlock {
	header: Bytes,
	body: Option<Bytes>,
	receipts: Option<Bytes>,
	receipts_root: H256,
}

/// Block with optional receipt
pub struct BlockAndReceipts {
	/// Block data.
	pub block: Bytes,
	/// Block receipts RLP list.
	pub receipts: Option<Bytes>,
}

impl HeapSizeOf for SyncBlock {
	fn heap_size_of_children(&self) -> usize {
		self.header.heap_size_of_children() + self.body.heap_size_of_children()
	}
}

/// Used to identify header by transactions and uncles hashes
#[derive(Eq, PartialEq, Hash)]
struct HeaderId {
	transactions_root: H256,
	uncles: H256
}

/// A collection of blocks and subchain pointers being downloaded. This keeps track of
/// which headers/bodies need to be downloaded, which are being downloaded and also holds
/// the downloaded blocks.
#[derive(Default)]
pub struct BlockCollection {
	/// Does this collection need block receipts.
	need_receipts: bool,
	/// Heads of subchains to download
	heads: Vec<H256>,
	/// Downloaded blocks.
	blocks: HashMap<H256, SyncBlock>,
	/// Downloaded blocks by parent.
	parents: HashMap<H256, H256>,
	/// Used to map body to header.
	header_ids: HashMap<HeaderId, H256>,
	/// Used to map receipts root to headers.
	receipt_ids: HashMap<H256, SmallHashVec>,
	/// First block in `blocks`.
	head: Option<H256>,
	/// Set of block header hashes being downloaded
	downloading_headers: HashSet<H256>,
	/// Set of block bodies being downloaded identified by block hash.
	downloading_bodies: HashSet<H256>,
	/// Set of block receipts being downloaded identified by receipt root.
	downloading_receipts: HashSet<H256>,
}

impl BlockCollection {
	/// Create a new instance.
	pub fn new(download_receipts: bool) -> BlockCollection {
		BlockCollection {
			need_receipts: download_receipts,
			blocks: HashMap::new(),
			header_ids: HashMap::new(),
			receipt_ids: HashMap::new(),
			heads: Vec::new(),
			parents: HashMap::new(),
			head: None,
			downloading_headers: HashSet::new(),
			downloading_bodies: HashSet::new(),
			downloading_receipts: HashSet::new(),
		}
	}

	/// Clear everything.
	pub fn clear(&mut self) {
		self.blocks.clear();
		self.parents.clear();
		self.header_ids.clear();
		self.receipt_ids.clear();
		self.heads.clear();
		self.head = None;
		self.downloading_headers.clear();
		self.downloading_bodies.clear();
		self.downloading_receipts.clear();
	}

	/// Reset collection for a new sync round with given subchain block hashes.
	pub fn reset_to(&mut self, hashes: Vec<H256>) {
		self.clear();
		self.heads = hashes;
	}

	/// Insert a set of headers into collection and advance subchain head pointers.
	pub fn insert_headers(&mut self, headers: Vec<Bytes>) {
		for h in headers {
			if let Err(e) =  self.insert_header(h) {
				trace!(target: "sync", "Ignored invalid header: {:?}", e);
			}
		}
		self.update_heads();
	}

	/// Insert a collection of block bodies for previously downloaded headers.
	pub fn insert_bodies(&mut self, bodies: Vec<Bytes>) -> usize {
		let mut inserted = 0;
		for b in bodies {
			if let Err(e) =  self.insert_body(b) {
				trace!(target: "sync", "Ignored invalid body: {:?}", e);
			} else {
				inserted += 1;
			}
		}
		inserted
	}

	/// Insert a collection of block receipts for previously downloaded headers.
	pub fn insert_receipts(&mut self, receipts: Vec<Bytes>) -> usize {
		if !self.need_receipts {
			return 0;
		}
		let mut inserted = 0;
		for r in receipts {
			if let Err(e) =  self.insert_receipt(r) {
				trace!(target: "sync", "Ignored invalid receipt: {:?}", e);
			} else {
				inserted += 1;
			}
		}
		inserted
	}

	/// Returns a set of block hashes that require a body download. The returned set is marked as being downloaded.
	pub fn needed_bodies(&mut self, count: usize, _ignore_downloading: bool) -> Vec<H256> {
		if self.head.is_none() {
			return Vec::new();
		}
		let mut needed_bodies: Vec<H256> = Vec::new();
		let mut head = self.head;
		while head.is_some() && needed_bodies.len() < count {
			head = self.parents.get(&head.unwrap()).cloned();
			if let Some(head) = head {
				match self.blocks.get(&head) {
					Some(block) if block.body.is_none() && !self.downloading_bodies.contains(&head) => {
						self.downloading_bodies.insert(head.clone());
						needed_bodies.push(head.clone());
					}
					_ => (),
				}
			}
		}
		for h in self.header_ids.values() {
			if needed_bodies.len() >= count {
				break;
			}
			if !self.downloading_bodies.contains(h) {
				needed_bodies.push(h.clone());
				self.downloading_bodies.insert(h.clone());
			}
		}
		needed_bodies
	}


	/// Returns a set of block hashes that require a receipt download. The returned set is marked as being downloaded.
	pub fn needed_receipts(&mut self, count: usize, _ignore_downloading: bool) -> Vec<H256> {
		if self.head.is_none() || !self.need_receipts {
			return Vec::new();
		}
		let mut needed_receipts: Vec<H256> = Vec::new();
		let mut head = self.head;
		while head.is_some() && needed_receipts.len() < count {
			head = self.parents.get(&head.unwrap()).cloned();
			if let Some(head) = head {
				match self.blocks.get(&head) {
					Some(block) => {
						if block.receipts.is_none() && !self.downloading_receipts.contains(&block.receipts_root) {
							self.downloading_receipts.insert(block.receipts_root);
							needed_receipts.push(head.clone());
						}
					}
					_ => (),
				}
			}
		}
		// If there are multiple blocks per receipt, only request one of them.
		for (root, h) in self.receipt_ids.iter().map(|(root, hashes)| (root, hashes[0])) {
			if needed_receipts.len() >= count {
				break;
			}
			if !self.downloading_receipts.contains(root) {
				needed_receipts.push(h.clone());
				self.downloading_receipts.insert(*root);
			}
		}
		needed_receipts
	}

	/// Returns a set of block hashes that require a header download. The returned set is marked as being downloaded.
	pub fn needed_headers(&mut self, count: usize, ignore_downloading: bool) -> Option<(H256, usize)> {
		// find subchain to download
		let mut download = None;
		{
			for h in &self.heads {
				if ignore_downloading || !self.downloading_headers.contains(h) {
					self.downloading_headers.insert(h.clone());
					download = Some(h.clone());
					break;
				}
			}
		}
		download.map(|h| (h, count))
	}

	/// Unmark header as being downloaded.
	pub fn clear_header_download(&mut self, hash: &H256) {
		self.downloading_headers.remove(hash);
	}

	/// Unmark block body as being downloaded.
	pub fn clear_body_download(&mut self, hashes: &[H256]) {
		for h in hashes {
			self.downloading_bodies.remove(h);
		}
	}

	/// Unmark block receipt as being downloaded.
	pub fn clear_receipt_download(&mut self, hashes: &[H256]) {
		for h in hashes {
			if let Some(ref block) = self.blocks.get(h) {
				self.downloading_receipts.remove(&block.receipts_root);
			}
		}
	}

	/// Get a valid chain of blocks ordered in descending order and ready for importing into blockchain.
	pub fn drain(&mut self) -> Vec<BlockAndReceipts> {
		if self.blocks.is_empty() || self.head.is_none() {
			return Vec::new();
		}

		let mut drained = Vec::new();
		let mut hashes = Vec::new();
		{
			let mut blocks = Vec::new();
			let mut head = self.head;
			while let Some(h) = head {
				head = self.parents.get(&h).cloned();
				if let Some(head) = head {
					match self.blocks.get(&head) {
						Some(block) if block.body.is_some() && (!self.need_receipts || block.receipts.is_some()) => {
							blocks.push(block);
							hashes.push(head);
							self.head = Some(head);
						}
						_ => break,
					}
				}
			}

			for block in blocks {
				let body = view!(BodyView, block.body.as_ref().expect("blocks contains only full blocks; qed"));
				let header = view!(HeaderView, &block.header);
				let block_view = Block::new_from_header_and_body(&header, &body);
				drained.push(BlockAndReceipts {
					block: block_view.rlp().as_raw().to_vec(),
					receipts: block.receipts.clone(),
				});
			}
		}
		for h in hashes {
			self.blocks.remove(&h);
		}
		trace!(target: "sync", "Drained {} blocks, new head :{:?}", drained.len(), self.head);
		drained
	}

	/// Check if the collection is empty. We consider the syncing round complete once
	/// there is no block data left and only a single or none head pointer remains.
	pub fn is_empty(&self) -> bool {
		self.heads.len() == 0 || (self.heads.len() == 1 && self.head.map_or(false, |h| h == self.heads[0]))
	}

	/// Check if collection contains a block header.
	pub fn contains(&self, hash: &H256) -> bool {
		self.blocks.contains_key(hash)
	}

	/// Check if collection contains a block header.
	pub fn contains_head(&self, hash: &H256) -> bool {
		self.heads.contains(hash)
	}

	/// Return used heap size.
	pub fn heap_size(&self) -> usize {
		self.heads.heap_size_of_children()
			+ self.blocks.heap_size_of_children()
			+ self.parents.heap_size_of_children()
			+ self.header_ids.heap_size_of_children()
			+ self.downloading_headers.heap_size_of_children()
			+ self.downloading_bodies.heap_size_of_children()
	}

	/// Check if given block hash is marked as being downloaded.
	pub fn is_downloading(&self, hash: &H256) -> bool {
		self.downloading_headers.contains(hash) || self.downloading_bodies.contains(hash)
	}

	fn insert_body(&mut self, b: Bytes) -> Result<(), network::Error> {
		let header_id = {
			let body = Rlp::new(&b);
			let tx = body.at(0)?;
			let tx_root = ordered_trie_root(tx.iter().map(|r| r.as_raw()));
			let uncles = keccak(body.at(1)?.as_raw());
			HeaderId {
				transactions_root: tx_root,
				uncles: uncles
			}
		};

		match self.header_ids.get(&header_id).cloned() {
			Some(h) => {
				self.header_ids.remove(&header_id);
				self.downloading_bodies.remove(&h);
				match self.blocks.get_mut(&h) {
					Some(ref mut block) => {
						trace!(target: "sync", "Got body {}", h);
						block.body = Some(b);
						Ok(())
					},
					None => {
						warn!("Got body with no header {}", h);
						Err(network::ErrorKind::BadProtocol.into())
					}
				}
			}
			None => {
				trace!(target: "sync", "Ignored unknown/stale block body. tx_root = {:?}, uncles = {:?}", header_id.transactions_root, header_id.uncles);
				Err(network::ErrorKind::BadProtocol.into())
			}
		}
	}

	fn insert_receipt(&mut self, r: Bytes) -> Result<(), network::Error> {
		let receipt_root = {
			let receipts = Rlp::new(&r);
			ordered_trie_root(receipts.iter().map(|r| r.as_raw()))
		};
		self.downloading_receipts.remove(&receipt_root);
		match self.receipt_ids.entry(receipt_root) {
			Entry::Occupied(entry) => {
				for h in entry.remove() {
					match self.blocks.get_mut(&h) {
						Some(ref mut block) => {
							trace!(target: "sync", "Got receipt {}", h);
							block.receipts = Some(r.clone());
						},
						None => {
							warn!("Got receipt with no header {}", h);
							return Err(network::ErrorKind::BadProtocol.into())
						}
					}
				}
				Ok(())
			}
			_ => {
				trace!(target: "sync", "Ignored unknown/stale block receipt {:?}", receipt_root);
				Err(network::ErrorKind::BadProtocol.into())
			}
		}
	}

	fn insert_header(&mut self, header: Bytes) -> Result<H256, DecoderError> {
		let info: BlockHeader = Rlp::new(&header).as_val()?;
		let hash = info.hash();
		if self.blocks.contains_key(&hash) {
			return Ok(hash);
		}
		match self.head {
			None if hash == self.heads[0] => {
				trace!(target: "sync", "New head {}", hash);
				self.head = Some(info.parent_hash().clone());
			},
			_ => ()
		}

		let mut block = SyncBlock {
			header: header,
			body: None,
			receipts: None,
			receipts_root: H256::new(),
		};
		let header_id = HeaderId {
			transactions_root: info.transactions_root().clone(),
			uncles: info.uncles_hash().clone(),
		};
		if header_id.transactions_root == KECCAK_NULL_RLP && header_id.uncles == KECCAK_EMPTY_LIST_RLP {
			// empty body, just mark as downloaded
			let mut body_stream = RlpStream::new_list(2);
			body_stream.append_raw(&::rlp::EMPTY_LIST_RLP, 1);
			body_stream.append_raw(&::rlp::EMPTY_LIST_RLP, 1);
			block.body = Some(body_stream.out());
		}
		else {
			trace!("Queueing body tx_root = {:?}, uncles = {:?}, block = {:?}, number = {}", header_id.transactions_root, header_id.uncles, hash, info.number());
			self.header_ids.insert(header_id, hash.clone());
		}
		if self.need_receipts {
			let receipt_root = info.receipts_root().clone();
			if receipt_root == KECCAK_NULL_RLP {
				let receipts_stream = RlpStream::new_list(0);
				block.receipts = Some(receipts_stream.out());
			} else {
				self.receipt_ids.entry(receipt_root).or_insert_with(|| SmallHashVec::new()).push(hash.clone());
			}
			block.receipts_root = receipt_root;
		}

		self.parents.insert(info.parent_hash().clone(), hash.clone());
		self.blocks.insert(hash.clone(), block);
		trace!(target: "sync", "New header: {:x}", hash);
		Ok(hash)
	}

	// update subchain headers
	fn update_heads(&mut self) {
		let mut new_heads = Vec::new();
		let old_subchains: HashSet<_> = { self.heads.iter().cloned().collect() };
		for s in self.heads.drain(..) {
			let mut h = s.clone();
			if !self.blocks.contains_key(&h) {
				new_heads.push(h);
				continue;
			}
			loop {
				match self.parents.get(&h) {
					Some(next) => {
						h = next.clone();
						if old_subchains.contains(&h) {
							trace!(target: "sync", "Completed subchain {:?}", s);
							break; // reached head of the other subchain, merge by not adding
						}
					},
					_ => {
						new_heads.push(h);
						break;
					}
				}
			}
		}
		self.heads = new_heads;
	}
}

#[cfg(test)]
mod test {
	use super::BlockCollection;
	use ethcore::client::{TestBlockChainClient, EachBlockWith, BlockId, BlockChainClient};
	use ethcore::views::HeaderView;
	use ethcore::header::BlockNumber;
	use rlp::*;

	fn is_empty(bc: &BlockCollection) -> bool {
		bc.heads.is_empty() &&
		bc.blocks.is_empty() &&
		bc.parents.is_empty() &&
		bc.header_ids.is_empty() &&
		bc.head.is_none() &&
		bc.downloading_headers.is_empty() &&
		bc.downloading_bodies.is_empty()
	}

	#[test]
	fn create_clear() {
		let mut bc = BlockCollection::new(false);
		assert!(is_empty(&bc));
		let client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Nothing);
		let hashes = (0 .. 100).map(|i| (&client as &BlockChainClient).block_hash(BlockId::Number(i)).unwrap()).collect();
		bc.reset_to(hashes);
		assert!(!is_empty(&bc));
		bc.clear();
		assert!(is_empty(&bc));
	}

	#[test]
	fn insert_headers() {
		let mut bc = BlockCollection::new(false);
		assert!(is_empty(&bc));
		let client = TestBlockChainClient::new();
		let nblocks = 200;
		client.add_blocks(nblocks, EachBlockWith::Nothing);
		let blocks: Vec<_> = (0..nblocks)
			.map(|i| (&client as &BlockChainClient).block(BlockId::Number(i as BlockNumber)).unwrap().into_inner())
			.collect();
		let headers: Vec<_> = blocks.iter().map(|b| Rlp::new(b).at(0).unwrap().as_raw().to_vec()).collect();
		let hashes: Vec<_> = headers.iter().map(|h| view!(HeaderView, h).hash()).collect();
		let heads: Vec<_> = hashes.iter().enumerate().filter_map(|(i, h)| if i % 20 == 0 { Some(h.clone()) } else { None }).collect();
		bc.reset_to(heads);
		assert!(!bc.is_empty());
		assert_eq!(hashes[0], bc.heads[0]);
		assert!(bc.needed_bodies(1, false).is_empty());
		assert!(!bc.contains(&hashes[0]));
		assert!(!bc.is_downloading(&hashes[0]));

		let (h, n) = bc.needed_headers(6, false).unwrap();
		assert!(bc.is_downloading(&hashes[0]));
		assert_eq!(hashes[0], h);
		assert_eq!(n, 6);
		assert_eq!(bc.downloading_headers.len(), 1);
		assert!(bc.drain().is_empty());

		bc.insert_headers(headers[0..6].to_vec());
		assert_eq!(hashes[5], bc.heads[0]);
		for h in &hashes[0..6] {
			bc.clear_header_download(h)
		}
		assert_eq!(bc.downloading_headers.len(), 0);
		assert!(!bc.is_downloading(&hashes[0]));
		assert!(bc.contains(&hashes[0]));

		assert_eq!(&bc.drain().into_iter().map(|b| b.block).collect::<Vec<_>>()[..], &blocks[0..6]);
		assert!(!bc.contains(&hashes[0]));
		assert_eq!(hashes[5], bc.head.unwrap());

		let (h, _) = bc.needed_headers(6, false).unwrap();
		assert_eq!(hashes[5], h);
		let (h, _) = bc.needed_headers(6, false).unwrap();
		assert_eq!(hashes[20], h);
		bc.insert_headers(headers[10..16].to_vec());
		assert!(bc.drain().is_empty());
		bc.insert_headers(headers[5..10].to_vec());
		assert_eq!(&bc.drain().into_iter().map(|b| b.block).collect::<Vec<_>>()[..], &blocks[6..16]);
		assert_eq!(hashes[15], bc.heads[0]);

		bc.insert_headers(headers[15..].to_vec());
		bc.drain();
		assert!(bc.is_empty());
	}

	#[test]
	fn insert_headers_with_gap() {
		let mut bc = BlockCollection::new(false);
		assert!(is_empty(&bc));
		let client = TestBlockChainClient::new();
		let nblocks = 200;
		client.add_blocks(nblocks, EachBlockWith::Nothing);
		let blocks: Vec<_> = (0..nblocks)
			.map(|i| (&client as &BlockChainClient).block(BlockId::Number(i as BlockNumber)).unwrap().into_inner())
			.collect();
		let headers: Vec<_> = blocks.iter().map(|b| Rlp::new(b).at(0).unwrap().as_raw().to_vec()).collect();
		let hashes: Vec<_> = headers.iter().map(|h| view!(HeaderView, h).hash()).collect();
		let heads: Vec<_> = hashes.iter().enumerate().filter_map(|(i, h)| if i % 20 == 0 { Some(h.clone()) } else { None }).collect();
		bc.reset_to(heads);

		bc.insert_headers(headers[2..22].to_vec());
		assert_eq!(hashes[0], bc.heads[0]);
		assert_eq!(hashes[21], bc.heads[1]);
		assert!(bc.head.is_none());
		bc.insert_headers(headers[0..2].to_vec());
		assert!(bc.head.is_some());
		assert_eq!(hashes[21], bc.heads[0]);
	}

	#[test]
	fn insert_headers_no_gap() {
		let mut bc = BlockCollection::new(false);
		assert!(is_empty(&bc));
		let client = TestBlockChainClient::new();
		let nblocks = 200;
		client.add_blocks(nblocks, EachBlockWith::Nothing);
		let blocks: Vec<_> = (0..nblocks)
			.map(|i| (&client as &BlockChainClient).block(BlockId::Number(i as BlockNumber)).unwrap().into_inner())
			.collect();
		let headers: Vec<_> = blocks.iter().map(|b| Rlp::new(b).at(0).unwrap().as_raw().to_vec()).collect();
		let hashes: Vec<_> = headers.iter().map(|h| view!(HeaderView, h).hash()).collect();
		let heads: Vec<_> = hashes.iter().enumerate().filter_map(|(i, h)| if i % 20 == 0 { Some(h.clone()) } else { None }).collect();
		bc.reset_to(heads);

		bc.insert_headers(headers[1..2].to_vec());
		assert!(bc.drain().is_empty());
		bc.insert_headers(headers[0..1].to_vec());
		assert_eq!(bc.drain().len(), 2);
	}
}

