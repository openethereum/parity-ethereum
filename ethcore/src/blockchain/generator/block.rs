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

use rlp::*;
use bigint::hash::{H256, H2048};
use bytes::Bytes;
use header::Header;
use transaction::SignedTransaction;

use super::fork::Forkable;
use super::bloom::WithBloom;
use super::complete::CompleteBlock;
use super::transaction::WithTransaction;

/// Helper structure, used for encoding blocks.
#[derive(Default)]
pub struct Block {
	pub header: Header,
	pub transactions: Vec<SignedTransaction>,
	pub uncles: Vec<Header>
}

impl Encodable for Block {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(3);
		s.append(&self.header);
		s.append_list(&self.transactions);
		s.append_list(&self.uncles);
	}
}

impl Forkable for Block {
	fn fork(mut self, fork_number: usize) -> Self where Self: Sized {
		let difficulty = self.header.difficulty().clone() - fork_number.into();
		self.header.set_difficulty(difficulty);
		self
	}
}

impl WithBloom for Block {
	fn with_bloom(mut self, bloom: H2048) -> Self where Self: Sized {
		self.header.set_log_bloom(bloom);
		self
	}
}

impl WithTransaction for Block {
	fn with_transaction(mut self, transaction: SignedTransaction) -> Self where Self: Sized {
		self.transactions.push(transaction);
		self
	}
}

impl CompleteBlock for Block {
	fn complete(mut self, parent_hash: H256) -> Bytes {
		self.header.set_parent_hash(parent_hash);
		encode(&self).into_vec()
	}
}
