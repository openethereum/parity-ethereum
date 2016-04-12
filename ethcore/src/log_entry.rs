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

//! Block log.

use util::*;
use basic_types::LogBloom;
use header::BlockNumber;
use ethjson;

/// A record of execution for a `LOG` operation.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
	/// The address of the contract executing at the point of the `LOG` operation.
	pub address: Address,
	/// The topics associated with the `LOG` operation.
	pub topics: Vec<H256>,
	/// The data associated with the `LOG` operation.
	pub data: Bytes,
}

impl Encodable for LogEntry {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(3);
		s.append(&self.address);
		s.append(&self.topics);
		s.append(&self.data);
	}
}

impl Decodable for LogEntry {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = decoder.as_rlp();
		let entry = LogEntry {
			address: try!(d.val_at(0)),
			topics: try!(d.val_at(1)),
			data: try!(d.val_at(2)),
		};
		Ok(entry)
	}
}

impl HeapSizeOf for LogEntry {
	fn heap_size_of_children(&self) -> usize {
		self.topics.heap_size_of_children() + self.data.heap_size_of_children()
	}
}

impl LogEntry {
	/// Calculates the bloom of this log entry.
	pub fn bloom(&self) -> LogBloom {
		self.topics.iter().fold(LogBloom::from_bloomed(&self.address.sha3()), |b, t| b.with_bloomed(&t.sha3()))
	}
}

impl From<ethjson::state::Log> for LogEntry {
	fn from(l: ethjson::state::Log) -> Self {
		LogEntry {
			address: l.address.into(),
			topics: l.topics.into_iter().map(Into::into).collect(),
			data: l.data.into(),
		}
	}
}

/// Log localized in a blockchain.
#[derive(Default, Debug, PartialEq, Clone)]
pub struct LocalizedLogEntry {
	/// Plain log entry.
	pub entry: LogEntry,
	/// Block in which this log was created.
	pub block_hash: H256,
	/// Block number.
	pub block_number: BlockNumber,
	/// Hash of transaction in which this log was created.
	pub transaction_hash: H256,
	/// Index of transaction within block.
	pub transaction_index: usize,
	/// Log position in the block.
	pub log_index: usize,
}

impl Deref for LocalizedLogEntry {
	type Target = LogEntry;

	fn deref(&self) -> &Self::Target {
		&self.entry
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use super::LogEntry;

	#[test]
	fn test_empty_log_bloom() {
		let bloom = H2048::from_str("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let log = LogEntry {
			address: address,
			topics: vec![],
			data: vec![]
		};
		assert_eq!(log.bloom(), bloom);
	}
}
