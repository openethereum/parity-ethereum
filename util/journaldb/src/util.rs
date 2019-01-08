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

use ethereum_types::H256;
use rlp::{RlpStream, Encodable, Rlp, DecoderError};

const PADDING : [u8; 10] = [ 0u8; 10 ];

pub struct DatabaseKey {
	pub era: u64,
	pub index: usize,
}

impl Encodable for DatabaseKey {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(3);
		s.append(&self.era);
		s.append(&self.index);
		s.append(&&PADDING[..]);
	}
}

pub struct DatabaseValueView<'a> {
	rlp: Rlp<'a>,
}

impl<'a> DatabaseValueView<'a> {
	pub fn from_rlp(data: &'a [u8]) -> Self {
		DatabaseValueView {
			rlp: Rlp::new(data),
		}
	}

	#[inline]
	pub fn id(&self) -> Result<H256, DecoderError> {
		self.rlp.val_at(0)
	}

	#[inline]
	pub fn inserts(&self) -> Result<Vec<H256>, DecoderError> {
		self.rlp.list_at(1)
	}

	#[inline]
	pub fn deletes(&self) -> Result<Vec<H256>, DecoderError> {
		self.rlp.list_at(2)
	}
}

pub struct DatabaseValueRef<'a> {
	pub id: &'a H256,
	pub inserts: &'a [H256],
	pub deletes: &'a [H256],
}

impl<'a> Encodable for DatabaseValueRef<'a> {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(3);
		s.append(self.id);
		s.append_list(self.inserts);
		s.append_list(self.deletes);
	}
}
