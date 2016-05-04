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

use bloomchain::group as bc;
use util::rlp::*;
use util::HeapSizeOf;
use super::Bloom;

/// Represents group of X consecutive blooms.
#[derive(Clone)]
pub struct BloomGroup {
	blooms: Vec<Bloom>,
}

impl From<bc::BloomGroup> for BloomGroup {
	fn from(group: bc::BloomGroup) -> Self {
		let blooms = group.blooms
			.into_iter()
			.map(From::from)
			.collect();

		BloomGroup {
			blooms: blooms
		}
	}
}

impl Into<bc::BloomGroup> for BloomGroup {
	fn into(self) -> bc::BloomGroup {
		let blooms = self.blooms
			.into_iter()
			.map(Into::into)
			.collect();

		bc::BloomGroup {
			blooms: blooms
		}
	}
}

impl Decodable for BloomGroup {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let blooms = try!(Decodable::decode(decoder));
		let group = BloomGroup {
			blooms: blooms
		};
		Ok(group)
	}
}

impl Encodable for BloomGroup {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.blooms, s)
	}
}

impl HeapSizeOf for BloomGroup {
	fn heap_size_of_children(&self) -> usize {
		self.blooms.heap_size_of_children()
	}
}
