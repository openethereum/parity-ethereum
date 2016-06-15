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

use block::Block;
use header::Header;

use views::BlockView;
use util::rlp::{DecoderError, RlpStream, Stream, UntrustedRlp, View};
use util::{Bytes, H256};

const HEADER_FIELDS: usize = 11;
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

		// 11 header fields, unknown amount of seal fields, and 2 block fields.
		let mut stream = RlpStream::new_list(
			HEADER_FIELDS +
			seal_fields.len() +
			BLOCK_FIELDS
		);

		// write header values.
		stream
			.append(&header.uncles_hash())
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

		// write seal fields.
		for field in seal_fields {
			stream.append_raw(&field, 1);
		}

		// write block values.
		stream.append(&block_view.transactions()).append(&block_view.uncles());

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
			uncles_hash: try!(rlp.val_at(0)),
			author: try!(rlp.val_at(1)),
			state_root: try!(rlp.val_at(2)),
			transactions_root: try!(rlp.val_at(3)),
			receipts_root: try!(rlp.val_at(4)),
			log_bloom: try!(rlp.val_at(5)),
			difficulty: try!(rlp.val_at(6)),
			number: number,
			gas_limit: try!(rlp.val_at(7)),
			gas_used: try!(rlp.val_at(8)),
			timestamp: try!(rlp.val_at(9)),
			extra_data: try!(rlp.val_at(10)),
			..Default::default()
		};

		let seal: Vec<Bytes> = try!(rlp.val_at(11));

		header.set_seal(seal);

		Ok(Block {
			header: header,
			transactions: try!(rlp.val_at(12)),
			uncles: try!(rlp.val_at(13)),
		})
	}
}