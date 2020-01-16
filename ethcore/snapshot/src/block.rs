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

//! Block RLP compression.

use bytes::Bytes;
use common_types::{
	block::Block,
	header::Header,
	views::BlockView,
};
use ethereum_types::H256;
use keccak_hash::keccak;
use rlp::{DecoderError, RlpStream, Rlp};
use triehash::ordered_trie_root;

const HEADER_FIELDS: usize = 8;
const BLOCK_FIELDS: usize = 2;

/// Convenience type to convert raw RLP to and from blocks.
pub struct AbridgedBlock {
	rlp: Bytes,
}

impl AbridgedBlock {
	/// Create from rlp-compressed bytes. Does no verification.
	pub fn from_raw(compressed: Bytes) -> Self {
		AbridgedBlock {
			rlp: compressed,
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

		// 10 header fields, unknown number of seal fields, and 2 block fields.
		let mut stream = RlpStream::new_list(
			HEADER_FIELDS +
			seal_fields.len() +
			BLOCK_FIELDS
		);

		// write header values.
		stream
			.append(&header.author())
			.append(&header.state_root())
			.append(&header.log_bloom())
			.append(&header.difficulty())
			.append(&header.gas_limit())
			.append(&header.gas_used())
			.append(&header.timestamp())
			.append(&header.extra_data());

		// write block values.
		stream
			.append_list(&block_view.transactions())
			.append_list(&block_view.uncles());

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
	pub fn to_block(&self, parent_hash: H256, number: u64, receipts_root: H256) -> Result<Block, DecoderError> {
		let rlp = Rlp::new(&self.rlp);

		let mut header: Header = Default::default();
		header.set_parent_hash(parent_hash);
		header.set_author(rlp.val_at(0)?);
		header.set_state_root(rlp.val_at(1)?);
		header.set_log_bloom(rlp.val_at(2)?);
		header.set_difficulty(rlp.val_at(3)?);
		header.set_number(number);
		header.set_gas_limit(rlp.val_at(4)?);
		header.set_gas_used(rlp.val_at(5)?);
		header.set_timestamp(rlp.val_at(6)?);
		header.set_extra_data(rlp.val_at(7)?);

		let transactions = rlp.list_at(8)?;
		let uncles: Vec<Header> = rlp.list_at(9)?;

		header.set_transactions_root(ordered_trie_root(
			rlp.at(8)?.iter().map(|r| r.as_raw())
		));
		header.set_receipts_root(receipts_root);

		let mut uncles_rlp = RlpStream::new();
		uncles_rlp.append_list(&uncles);
		header.set_uncles_hash(keccak(uncles_rlp.as_raw()));

		let mut seal_fields = Vec::new();
		for i in (HEADER_FIELDS + BLOCK_FIELDS)..rlp.item_count()? {
			let seal_rlp = rlp.at(i)?;
			seal_fields.push(seal_rlp.as_raw().to_owned());
		}

		header.set_seal(seal_fields);

		Ok(Block { header, transactions, uncles })
	}
}
