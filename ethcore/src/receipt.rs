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

//! Receipt

use util::*;
use basic_types::LogBloom;
use header::BlockNumber;
use log_entry::{LogEntry, LocalizedLogEntry};

/// Information describing execution of a transaction.
#[derive(Default, Debug, Clone)]
pub struct Receipt {
	/// The state root after executing the transaction.
	pub state_root: H256,
	/// The total gas used in the block following execution of the transaction.
	pub gas_used: U256,
	/// The OR-wide combination of all logs' blooms for this transaction.
	pub log_bloom: LogBloom,
	/// The logs stemming from this transaction.
	pub logs: Vec<LogEntry>,
}

impl Receipt {
	/// Create a new receipt.
	pub fn new(state_root: H256, gas_used: U256, logs: Vec<LogEntry>) -> Receipt {
		Receipt {
			state_root: state_root,
			gas_used: gas_used,
			log_bloom: logs.iter().fold(LogBloom::new(), |mut b, l| { b = &b | &l.bloom(); b }), //TODO: use |= operator
			logs: logs,
		}
	}
}

impl Encodable for Receipt {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.state_root);
		s.append(&self.gas_used);
		s.append(&self.log_bloom);
		s.append(&self.logs);
	}
}

impl Decodable for Receipt {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let receipt = Receipt {
			state_root: try!(d.val_at(0)),
			gas_used: try!(d.val_at(1)),
			log_bloom: try!(d.val_at(2)),
			logs: try!(d.val_at(3)),
		};
		Ok(receipt)
	}
}

impl HeapSizeOf for Receipt {
	fn heap_size_of_children(&self) -> usize {
		self.logs.heap_size_of_children()
	}
}

/// Receipt with additional info.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalizedReceipt {
	/// Transaction hash.
	pub transaction_hash: H256,
	/// Transaction index.
	pub transaction_index: usize,
	/// Block hash.
	pub block_hash: H256,
	/// Block number.
	pub block_number: BlockNumber,
	/// Cumulative gas used.
	pub cumulative_gas_used: U256,
	/// Gas used.
	pub gas_used: U256,
	/// Contract address.
	pub contract_address: Option<Address>,
	/// Logs
	pub logs: Vec<LocalizedLogEntry>,
}

#[test]
fn test_basic() {
	let expected = FromHex::from_hex("f90162a02f697d671e9ae4ee24a43c4b0d7e15f1cb4ba6de1561120d43b9a4e8c4a8a6ee83040caeb9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000f838f794dcf421d093428b096ca501a7cd1a740855a7976fc0a00000000000000000000000000000000000000000000000000000000000000000").unwrap();
	let r = Receipt::new(
		x!("2f697d671e9ae4ee24a43c4b0d7e15f1cb4ba6de1561120d43b9a4e8c4a8a6ee"),
		x!(0x40cae),
		vec![LogEntry {
			address: x!("dcf421d093428b096ca501a7cd1a740855a7976f"),
			topics: vec![],
			data: vec![0u8; 32]
		}]
	);
	assert_eq!(&encode(&r)[..], &expected[..]);
}
