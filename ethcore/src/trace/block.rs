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

use util::rlp::*;
use basic_types::LogBloom;
use super::Trace;

/// Traces created by transactions from the same block.
#[derive(Clone)]
pub struct BlockTraces(Vec<Trace>);

impl From<Vec<Trace>> for BlockTraces {
	fn from(traces: Vec<Trace>) -> Self {
		BlockTraces(traces)
	}
}

impl Into<Vec<Trace>> for BlockTraces {
	fn into(self) -> Vec<Trace> {
		self.0
	}
}

impl Decodable for BlockTraces {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let traces = try!(Decodable::decode(decoder));
		let block_traces = BlockTraces(traces);
		Ok(block_traces)
	}
}

impl Encodable for BlockTraces {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.0, s)
	}
}

impl BlockTraces {
	/// Returns bloom of all traces in given block.
	pub fn bloom(&self) -> LogBloom {
		self.0.iter()
			.fold(LogBloom::default(), |acc, trace| acc | trace.bloom())
	}
}

