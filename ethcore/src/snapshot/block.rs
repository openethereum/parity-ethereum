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

//! Block RLP compression.

// TODO [rob] remove when BlockRebuilder done.
#![allow(dead_code)]

use block::Block;
use header::Header;

use views::BlockView;
use util::rlp::{DecoderError, RlpStream, Stream, UntrustedRlp, View};
use util::{Bytes, Hashable, H256};

const HEADER_FIELDS: usize = 10;
const BLOCK_FIELDS: usize = 2;

pub struct AbridgedBlock {
	rlp: Bytes,
}

impl AbridgedBlock {
	/// Create from a vector of bytes. Does no verification.
	pub fn from_raw(rlp: Bytes) -> Self {
		AbridgedBlock {
			rlp: rlp,
		}
	}

	/// Return the inner bytes.
	pub fn into_inner(self) -> Bytes {
		self.rlp
	}

	/// Given a full block view, trim out the parent hash and block number,
	/// producing new rlp.
	pub fn from_block_view(block_view: &BlockView) -> Self {
		let header = block_view.header_view();

		let seal_fields = header.seal();

		// 10 header fields, unknown amount of seal fields, and 2 block fields.
		let mut stream = RlpStream::new_list(
			HEADER_FIELDS +
			seal_fields.len() +
			BLOCK_FIELDS
		);

		// write header values.
		stream
			.append(&header.author())
			.append(&header.state_root())
			.append(&header.transactions_root())
			.append(&header.receipts_root())
			.append(&header.log_bloom())
			.append(&header.difficulty())
			.append(&header.gas_limit())
			.append(&header.gas_used())
			.append(&header.timestamp())
			.append(&header.extra_data());

		// write block values.
		stream.append(&block_view.transactions()).append(&block_view.uncles());

		// write seal fields.
		for field in seal_fields {
			stream.append_raw(&field, 1);
		}

		AbridgedBlock {
			rlp: stream.out(),
		}
	}

	/// Flesh out an abridged block view with the provided parent hash and block number.
	///
	/// Will fail if contains invalid rlp.
	pub fn to_block(&self, parent_hash: H256, number: u64) -> Result<Block, DecoderError> {
		let rlp = UntrustedRlp::new(&self.rlp);

		let mut header = Header {
			parent_hash: parent_hash,
			author: try!(rlp.val_at(0)),
			state_root: try!(rlp.val_at(1)),
			transactions_root: try!(rlp.val_at(2)),
			receipts_root: try!(rlp.val_at(3)),
			log_bloom: try!(rlp.val_at(4)),
			difficulty: try!(rlp.val_at(5)),
			number: number,
			gas_limit: try!(rlp.val_at(6)),
			gas_used: try!(rlp.val_at(7)),
			timestamp: try!(rlp.val_at(8)),
			extra_data: try!(rlp.val_at(9)),
			..Default::default()
		};
		let transactions = try!(rlp.val_at(10));
		let uncles: Vec<Header> = try!(rlp.val_at(11));

		// iterator-based approach is cleaner but doesn't work w/ try.
		let seal = {
			let mut seal = Vec::new();

			for i in 12..rlp.item_count() {
				seal.push(try!(rlp.val_at(i)));
			}

			seal
		};

		header.set_seal(seal);

		let uncle_bytes = uncles.iter()
			.fold(RlpStream::new_list(uncles.len()), |mut s, u| {
				s.append_raw(&u.rlp(::basic_types::Seal::With), 1);
				s
			}).out();
		header.uncles_hash = uncle_bytes.sha3();

		Ok(Block {
			header: header,
			transactions: transactions,
			uncles: uncles,
		})
	}
}