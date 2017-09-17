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

use bigint::hash::H256;
use bytes::Bytes;
use views::BlockView;

#[derive(Default, Clone)]
pub struct BlockFinalizer {
	parent_hash: H256
}

impl BlockFinalizer {
	pub fn fork(&self) -> Self {
		self.clone()
	}
}

pub trait CompleteBlock {
	fn complete(self, parent_hash: H256) -> Bytes;
}

pub struct Complete<'a, I> where I: 'a {
	pub iter: &'a mut I,
	pub finalizer: &'a mut BlockFinalizer,
}

impl<'a, I> Iterator for Complete<'a, I> where I: Iterator, <I as Iterator>::Item: CompleteBlock {
	type Item = Bytes;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|item| {
			let rlp = item.complete(self.finalizer.parent_hash.clone());
			self.finalizer.parent_hash = BlockView::new(&rlp).header_view().hash();
			rlp
		})
	}
}
