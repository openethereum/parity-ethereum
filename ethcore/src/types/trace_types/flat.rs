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

//! Flat trace module

use std::collections::VecDeque;
use std::mem;
use ipc::binary::BinaryConvertError;
use util::rlp::*;
use util::HeapSizeOf;
use basic_types::LogBloom;
use super::trace::{Action, Res};

/// Trace localized in vector of traces produced by a single transaction.
///
/// Parent and children indexes refer to positions in this vector.
#[derive(Debug, PartialEq, Clone, Binary)]
pub struct FlatTrace {
	/// Type of action performed by a transaction.
	pub action: Action,
	/// Result of this action.
	pub result: Res,
	/// Number of subtraces.
	pub subtraces: usize,
	/// Exact location of trace.
	///
	/// [index in root, index in first CALL, index in second CALL, ...]
	pub trace_address: VecDeque<usize>,
}

impl FlatTrace {
	/// Returns bloom of the trace.
	pub fn bloom(&self) -> LogBloom {
		self.action.bloom() | self.result.bloom()
	}
}

impl HeapSizeOf for FlatTrace {
	fn heap_size_of_children(&self) -> usize {
		self.trace_address.heap_size_of_children()
	}
}

impl Encodable for FlatTrace {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.action);
		s.append(&self.result);
		s.append(&self.subtraces);
		s.append(&self.trace_address.clone().into_iter().collect::<Vec<_>>());
	}
}

impl Decodable for FlatTrace {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let v: Vec<usize> = try!(d.val_at(3));
		let res = FlatTrace {
			action: try!(d.val_at(0)),
			result: try!(d.val_at(1)),
			subtraces: try!(d.val_at(2)),
			trace_address: v.into_iter().collect(),
		};

		Ok(res)
	}
}

/// Represents all traces produced by a single transaction.
#[derive(Debug, PartialEq, Clone)]
pub struct FlatTransactionTraces(Vec<FlatTrace>);

impl From<Vec<FlatTrace>> for FlatTransactionTraces {
	fn from(v: Vec<FlatTrace>) -> Self {
		FlatTransactionTraces(v)
	}
}

impl HeapSizeOf for FlatTransactionTraces {
	fn heap_size_of_children(&self) -> usize {
		self.0.heap_size_of_children()
	}
}

impl FlatTransactionTraces {
	/// Returns bloom of all traces in the collection.
	pub fn bloom(&self) -> LogBloom {
		self.0.iter().fold(Default::default(), | bloom, trace | bloom | trace.bloom())
	}
}

impl Encodable for FlatTransactionTraces {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&self.0);
	}
}

impl Decodable for FlatTransactionTraces {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		Ok(FlatTransactionTraces(try!(Decodable::decode(decoder))))
	}
}

impl Into<Vec<FlatTrace>> for FlatTransactionTraces {
	fn into(self) -> Vec<FlatTrace> {
		self.0
	}
}

/// Represents all traces produced by transactions in a single block.
#[derive(Debug, PartialEq, Clone)]
pub struct FlatBlockTraces(Vec<FlatTransactionTraces>);

impl HeapSizeOf for FlatBlockTraces {
	fn heap_size_of_children(&self) -> usize {
		self.0.heap_size_of_children()
	}
}

impl From<Vec<FlatTransactionTraces>> for FlatBlockTraces {
	fn from(v: Vec<FlatTransactionTraces>) -> Self {
		FlatBlockTraces(v)
	}
}

impl FlatBlockTraces {
	/// Returns bloom of all traces in the block.
	pub fn bloom(&self) -> LogBloom {
		self.0.iter().fold(Default::default(), | bloom, tx_traces | bloom | tx_traces.bloom())
	}
}

impl Encodable for FlatBlockTraces {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&self.0);
	}
}

impl Decodable for FlatBlockTraces {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		Ok(FlatBlockTraces(try!(Decodable::decode(decoder))))
	}
}

impl Into<Vec<FlatTransactionTraces>> for FlatBlockTraces {
	fn into(self) -> Vec<FlatTransactionTraces> {
		self.0
	}
}

#[cfg(test)]
mod tests {
	use super::{FlatBlockTraces, FlatTransactionTraces, FlatTrace};
	use trace::trace::{Action, Res, CallResult, Call};
	use types::executed::CallType;

	#[test]
	fn test_trace_serialization() {
		use util::rlp;

		let flat_trace = FlatTrace {
			action: Action::Call(Call {
				from: 1.into(),
				to: 2.into(),
				value: 3.into(),
				gas: 4.into(),
				input: vec![0x5],
				call_type: CallType::Call,
			}),
			result: Res::Call(CallResult {
				gas_used: 10.into(),
				output: vec![0x11, 0x12]
			}),
			trace_address: Default::default(),
			subtraces: 0,
		};

		let block_traces = FlatBlockTraces(vec![FlatTransactionTraces(vec![flat_trace])]);

		let encoded = rlp::encode(&block_traces);
		let decoded = rlp::decode(&encoded);
		assert_eq!(block_traces, decoded);
	}
}
