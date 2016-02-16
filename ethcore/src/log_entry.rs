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

use util::*;
use basic_types::LogBloom;

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

impl LogEntry {
	/// Create a new log entry.
	pub fn new(address: Address, topics: Vec<H256>, data: Bytes) -> LogEntry {
		LogEntry {
			address: address,
			topics: topics,
			data: data,
		}
	}

	/// Calculates the bloom of this log entry.
	pub fn bloom(&self) -> LogBloom {
		self.topics.iter().fold(LogBloom::from_bloomed(&self.address.sha3()), |b, t| b.with_bloomed(&t.sha3()))
	}
}

impl FromJson for LogEntry {
	/// Convert given JSON object to a LogEntry.
	fn from_json(json: &Json) -> LogEntry {
		// TODO: check bloom.
		LogEntry {
			address: xjson!(&json["address"]),
			topics: xjson!(&json["topics"]),
			data: xjson!(&json["data"]),
		}
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
		let log = LogEntry::new(address, vec![], vec![]);
		assert_eq!(log.bloom(), bloom);
	}
}
