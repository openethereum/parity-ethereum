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

use serde::{Serialize, Serializer};
use util::uint::*;

#[derive(Default, Debug, Serialize, PartialEq)]
pub struct SyncInfo {
	#[serde(rename="startingBlock")]
	pub starting_block: U256,
	#[serde(rename="currentBlock")]
	pub current_block: U256,
	#[serde(rename="highestBlock")]
	pub highest_block: U256,
}

#[derive(Debug, PartialEq)]
pub enum SyncStatus {
	Info(SyncInfo),
	None
}

impl Serialize for SyncStatus {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		match *self {
			SyncStatus::Info(ref info) => info.serialize(serializer),
			SyncStatus::None => false.serialize(serializer)
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::*;

	#[test]
	fn test_serialize_sync_info() {
		let t = SyncInfo::default();
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"startingBlock":"0x00","currentBlock":"0x00","highestBlock":"0x00"}"#);
	}

	#[test]
	fn test_serialize_sync_status() {
		let t = SyncStatus::None;
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, "false");

		let t = SyncStatus::Info(SyncInfo::default());
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"startingBlock":"0x00","currentBlock":"0x00","highestBlock":"0x00"}"#);
	}
}
