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
use ethcore::client::{BlockId};
use ethcore::header::{BlockNumber};
use ethereum_types::{H256};
use network::{PeerId};
use rlp::{Rlp, RlpStream};
use std::cmp;
use sync_io::SyncIo;

use super::{
	RlpResponseResult,
	BLOCK_HEADERS_PACKET,
	MAX_HEADERS_TO_SEND,
};

pub struct SyncSupplier {}

impl SyncSupplier {
	/// Respond to GetBlockHeaders request
	pub fn return_block_headers(io: &SyncIo, r: &Rlp, peer_id: PeerId) -> RlpResponseResult {
		// Packet layout:
		// [ block: { P , B_32 }, maxHeaders: P, skip: P, reverse: P in { 0 , 1 } ]
		let max_headers: usize = r.val_at(1)?;
		let skip: usize = r.val_at(2)?;
		let reverse: bool = r.val_at(3)?;
		let last = io.chain().chain_info().best_block_number;
		let number = if r.at(0)?.size() == 32 {
			// id is a hash
			let hash: H256 = r.val_at(0)?;
			trace!(target: "sync", "{} -> GetBlockHeaders (hash: {}, max: {}, skip: {}, reverse:{})", peer_id, hash, max_headers, skip, reverse);
			match io.chain().block_header(BlockId::Hash(hash)) {
				Some(hdr) => {
					let number = hdr.number().into();
					debug_assert_eq!(hdr.hash(), hash);

					if max_headers == 1 || io.chain().block_hash(BlockId::Number(number)) != Some(hash) {
						// Non canonical header or single header requested
						// TODO: handle single-step reverse hashchains of non-canon hashes
						trace!(target:"sync", "Returning single header: {:?}", hash);
						let mut rlp = RlpStream::new_list(1);
						rlp.append_raw(&hdr.into_inner(), 1);
						return Ok(Some((BLOCK_HEADERS_PACKET, rlp)));
					}
					number
				}
				None => return Ok(Some((BLOCK_HEADERS_PACKET, RlpStream::new_list(0)))) //no such header, return nothing
			}
		} else {
			trace!(target: "sync", "{} -> GetBlockHeaders (number: {}, max: {}, skip: {}, reverse:{})", peer_id, r.val_at::<BlockNumber>(0)?, max_headers, skip, reverse);
			r.val_at(0)?
		};

		let mut number = if reverse {
			cmp::min(last, number)
		} else {
			cmp::max(0, number)
		};
		let max_count = cmp::min(MAX_HEADERS_TO_SEND, max_headers);
		let mut count = 0;
		let mut data = Bytes::new();
		let inc = (skip + 1) as BlockNumber;
		let overlay = io.chain_overlay().read();

		while number <= last && count < max_count {
			if let Some(hdr) = overlay.get(&number) {
				trace!(target: "sync", "{}: Returning cached fork header", peer_id);
				data.extend_from_slice(hdr);
				count += 1;
			} else if let Some(hdr) = io.chain().block_header(BlockId::Number(number)) {
				data.append(&mut hdr.into_inner());
				count += 1;
			} else {
				// No required block.
				break;
			}
			if reverse {
				if number <= inc || number == 0 {
					break;
				}
				number -= inc;
			}
			else {
				number += inc;
			}
		}
		let mut rlp = RlpStream::new_list(count as usize);
		rlp.append_raw(&data, count as usize);
		trace!(target: "sync", "{} -> GetBlockHeaders: returned {} entries", peer_id, count);
		Ok(Some((BLOCK_HEADERS_PACKET, rlp)))
	}
}

#[cfg(test)]
mod test {
	use std::collections::{VecDeque};
	use tests::helpers::{TestIo};
	use tests::snapshot::TestSnapshotService;
	use ethereum_types::{H256};
	use parking_lot::RwLock;
	use bytes::Bytes;
	use rlp::{Rlp, RlpStream};
	use super::*;
	use ethcore::client::{BlockChainClient, EachBlockWith, TestBlockChainClient};

	#[test]
	fn return_block_headers() {
		use ethcore::views::HeaderView;
		fn make_hash_req(h: &H256, count: usize, skip: usize, reverse: bool) -> Bytes {
			let mut rlp = RlpStream::new_list(4);
			rlp.append(h);
			rlp.append(&count);
			rlp.append(&skip);
			rlp.append(&if reverse {1u32} else {0u32});
			rlp.out()
		}

		fn make_num_req(n: usize, count: usize, skip: usize, reverse: bool) -> Bytes {
			let mut rlp = RlpStream::new_list(4);
			rlp.append(&n);
			rlp.append(&count);
			rlp.append(&skip);
			rlp.append(&if reverse {1u32} else {0u32});
			rlp.out()
		}
		fn to_header_vec(rlp: ::chain::RlpResponseResult) -> Vec<Bytes> {
			Rlp::new(&rlp.unwrap().unwrap().1.out()).iter().map(|r| r.as_raw().to_vec()).collect()
		}

		let mut client = TestBlockChainClient::new();
		client.add_blocks(100, EachBlockWith::Nothing);
		let blocks: Vec<_> = (0 .. 100)
			.map(|i| (&client as &BlockChainClient).block(BlockId::Number(i as BlockNumber)).map(|b| b.into_inner()).unwrap()).collect();
		let headers: Vec<_> = blocks.iter().map(|b| Rlp::new(b).at(0).unwrap().as_raw().to_vec()).collect();
		let hashes: Vec<_> = headers.iter().map(|h| view!(HeaderView, h).hash()).collect();

		let queue = RwLock::new(VecDeque::new());
		let ss = TestSnapshotService::new();
		let io = TestIo::new(&mut client, &ss, &queue, None);

		let unknown: H256 = H256::new();
		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_hash_req(&unknown, 1, 0, false)), 0);
		assert!(to_header_vec(result).is_empty());
		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_hash_req(&unknown, 1, 0, true)), 0);
		assert!(to_header_vec(result).is_empty());

		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_hash_req(&hashes[2], 1, 0, true)), 0);
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_hash_req(&hashes[2], 1, 0, false)), 0);
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_hash_req(&hashes[50], 3, 5, false)), 0);
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[56].clone(), headers[62].clone()]);

		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_hash_req(&hashes[50], 3, 5, true)), 0);
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[44].clone(), headers[38].clone()]);

		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_num_req(2, 1, 0, true)), 0);
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_num_req(2, 1, 0, false)), 0);
		assert_eq!(to_header_vec(result), vec![headers[2].clone()]);

		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_num_req(50, 3, 5, false)), 0);
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[56].clone(), headers[62].clone()]);

		let result = SyncSupplier::return_block_headers(&io, &Rlp::new(&make_num_req(50, 3, 5, true)), 0);
		assert_eq!(to_header_vec(result), vec![headers[50].clone(), headers[44].clone(), headers[38].clone()]);
	}
}
