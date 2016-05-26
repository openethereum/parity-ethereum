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

use bloomchain as bc;
use util::rlp::*;
use util::HeapSizeOf;
use basic_types::LogBloom;

/// Helper structure representing bloom of the trace.
#[derive(Clone)]
pub struct Bloom(LogBloom);

impl From<LogBloom> for Bloom {
	fn from(bloom: LogBloom) -> Self {
		Bloom(bloom)
	}
}

impl From<bc::Bloom> for Bloom {
	fn from(bloom: bc::Bloom) -> Self {
		let bytes: [u8; 256] = bloom.into();
		Bloom(LogBloom::from(bytes))
	}
}

impl Into<bc::Bloom> for Bloom {
	fn into(self) -> bc::Bloom {
		let log = self.0;
		bc::Bloom::from(log.0)
	}
}

impl Decodable for Bloom {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		Decodable::decode(decoder).map(Bloom)
	}
}

impl Encodable for Bloom {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.0, s)
	}
}

impl HeapSizeOf for Bloom {
	fn heap_size_of_children(&self) -> usize {
		0
	}
}
