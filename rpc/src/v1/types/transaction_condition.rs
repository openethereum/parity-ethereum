// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use types::transaction;

/// Represents condition on minimum block number or block timestamp.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TransactionCondition {
	/// Valid at this minimum block number.
	#[serde(rename = "block")]
	Number(u64),
	/// Valid at given unix time.
	#[serde(rename = "time")]
	Timestamp(u64),
}

impl Into<transaction::Condition> for TransactionCondition {
	fn into(self) -> transaction::Condition {
		match self {
			TransactionCondition::Number(n) => transaction::Condition::Number(n),
			TransactionCondition::Timestamp(n) => transaction::Condition::Timestamp(n),
		}
	}
}

impl From<transaction::Condition> for TransactionCondition {
	fn from(condition: transaction::Condition) -> Self {
		match condition {
			transaction::Condition::Number(n) => TransactionCondition::Number(n),
			transaction::Condition::Timestamp(n) => TransactionCondition::Timestamp(n),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;

	#[test]
	fn condition_deserialization() {
		let s = r#"[{ "block": 51 }, { "time": 10 }]"#;
		let deserialized: Vec<TransactionCondition> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![TransactionCondition::Number(51), TransactionCondition::Timestamp(10)])
	}

	#[test]
	fn condition_into() {
		assert_eq!(transaction::Condition::Number(100), TransactionCondition::Number(100).into());
		assert_eq!(transaction::Condition::Timestamp(100), TransactionCondition::Timestamp(100).into());
	}
}
