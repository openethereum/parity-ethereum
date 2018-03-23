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

//! Flat trace module

use std::collections::VecDeque;
use rlp::{Rlp, RlpStream, Decodable, Encodable, DecoderError};
use heapsize::HeapSizeOf;
use ethereum_types::Bloom;
use super::trace::{Action, Res};

/// Trace localized in vector of traces produced by a single transaction.
///
/// Parent and children indexes refer to positions in this vector.
#[derive(Debug, PartialEq, Clone)]
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
	pub fn bloom(&self) -> Bloom {
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
		s.append_list::<usize, &usize>(&self.trace_address.iter().collect::<Vec<_>>());
	}
}

impl Decodable for FlatTrace {
	fn decode(d: &Rlp) -> Result<Self, DecoderError> {
		let v: Vec<usize> = d.list_at(3)?;
		let res = FlatTrace {
			action: d.val_at(0)?,
			result: d.val_at(1)?,
			subtraces: d.val_at(2)?,
			trace_address: v.into_iter().collect(),
		};

		Ok(res)
	}
}

/// Represents all traces produced by a single transaction.
#[derive(Debug, PartialEq, Clone, RlpEncodableWrapper, RlpDecodableWrapper)]
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
	pub fn bloom(&self) -> Bloom {
		self.0.iter().fold(Default::default(), | bloom, trace | bloom | trace.bloom())
	}
}

impl Into<Vec<FlatTrace>> for FlatTransactionTraces {
	fn into(self) -> Vec<FlatTrace> {
		self.0
	}
}

/// Represents all traces produced by transactions in a single block.
#[derive(Debug, PartialEq, Clone, Default, RlpEncodableWrapper, RlpDecodableWrapper)]
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
	pub fn bloom(&self) -> Bloom {
		self.0.iter().fold(Default::default(), | bloom, tx_traces | bloom | tx_traces.bloom())
	}
}

impl Into<Vec<FlatTransactionTraces>> for FlatBlockTraces {
	fn into(self) -> Vec<FlatTransactionTraces> {
		self.0
	}
}

#[cfg(test)]
mod tests {
	use rlp::*;
	use super::{FlatBlockTraces, FlatTransactionTraces, FlatTrace};
	use trace::trace::{Action, Res, CallResult, Call, Suicide, Reward};
	use evm::CallType;
	use trace::RewardType;

	#[test]
	fn encode_flat_transaction_traces() {
		let ftt = FlatTransactionTraces::from(Vec::new());

		let mut s = RlpStream::new_list(2);
		s.append(&ftt);
		assert!(!s.is_finished(), "List shouldn't finished yet");
		s.append(&ftt);
		assert!(s.is_finished(), "List should be finished now");
		s.out();
	}

	#[test]
	fn encode_flat_block_traces() {
		let fbt = FlatBlockTraces::from(Vec::new());

		let mut s = RlpStream::new_list(2);
		s.append(&fbt);
		assert!(!s.is_finished(), "List shouldn't finished yet");
		s.append(&fbt);
		assert!(s.is_finished(), "List should be finished now");
		s.out();
	}

	#[test]
	fn test_trace_serialization() {
		// block #51921

		let flat_trace = FlatTrace {
			action: Action::Call(Call {
				from: "8dda5e016e674683241bf671cced51e7239ea2bc".parse().unwrap(),
				to: "37a5e19cc2d49f244805d5c268c0e6f321965ab9".parse().unwrap(),
				value: "3627e8f712373c0000".parse().unwrap(),
				gas: 0x03e8.into(),
				input: vec![],
				call_type: CallType::Call,
			}),
			result: Res::Call(CallResult {
				gas_used: 0.into(),
				output: vec![],
			}),
			trace_address: Default::default(),
			subtraces: 0,
		};

		let flat_trace1 = FlatTrace {
			action: Action::Call(Call {
				from: "3d0768da09ce77d25e2d998e6a7b6ed4b9116c2d".parse().unwrap(),
				to: "412fda7643b37d436cb40628f6dbbb80a07267ed".parse().unwrap(),
				value: 0.into(),
				gas: 0x010c78.into(),
				input: vec![0x41, 0xc0, 0xe1, 0xb5],
				call_type: CallType::Call,
			}),
			result: Res::Call(CallResult {
				gas_used: 0x0127.into(),
				output: vec![],
			}),
			trace_address: Default::default(),
			subtraces: 1,
		};

		let flat_trace2 = FlatTrace {
			action: Action::Suicide(Suicide {
				address: "412fda7643b37d436cb40628f6dbbb80a07267ed".parse().unwrap(),
				balance: 0.into(),
				refund_address: "3d0768da09ce77d25e2d998e6a7b6ed4b9116c2d".parse().unwrap(),
			}),
			result: Res::None,
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
		};

		let flat_trace3 = FlatTrace {
			action: Action::Reward(Reward {
				author: "412fda7643b37d436cb40628f6dbbb80a07267ed".parse().unwrap(),
				value: 10.into(),
				reward_type: RewardType::Uncle,
			}),
			result: Res::None,
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
		};

		let flat_trace4 = FlatTrace {
			action: Action::Reward(Reward {
				author: "412fda7643b37d436cb40628f6dbbb80a07267ed".parse().unwrap(),
				value: 10.into(),
				reward_type: RewardType::Block,
			}),
			result: Res::None,
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
		};

		let block_traces = FlatBlockTraces(vec![
			FlatTransactionTraces(vec![flat_trace]),
			FlatTransactionTraces(vec![flat_trace1, flat_trace2]),
			FlatTransactionTraces(vec![flat_trace3, flat_trace4])
		]);

		let encoded = ::rlp::encode(&block_traces);
		let decoded = ::rlp::decode(&encoded);
		assert_eq!(block_traces, decoded);
	}
}
