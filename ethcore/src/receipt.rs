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
use log_entry::LogEntry;

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
			log_bloom: logs.iter().fold(LogBloom::new(), |mut b, l| {
				b |= &l.bloom();
				b
			}),
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


#[test]
fn test_basic() {
	let expected = FromHex::from_hex("f90162a02f697d671e9ae4ee24a43c4b0d7e15f1cb4ba6de1561120d43b9a4e8c4a8a6ee83040caeb9010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000f838f794dcf421d093428b096ca501a7cd1a740855a7976fc0a00000000000000000000000000000000000000000000000000000000000000000").unwrap();
	let r = Receipt::new(x!("2f697d671e9ae4ee24a43c4b0d7e15f1cb4ba6de1561120d43b9a4e8c4a8a6ee"),
	                     x!(0x40cae),
	                     vec![LogEntry::new(x!("dcf421d093428b096ca501a7cd1a740855a7976f"), vec![], vec![0u8; 32])]);
	assert_eq!(&encode(&r)[..], &expected[..]);
}
