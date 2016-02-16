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
use serde_json::Value;

#[derive(Debug)]
pub enum OptionalValue<T>
	where T: Serialize,
{
	Value(T),
	Null,
}

impl<T> Default for OptionalValue<T>
    where T: Serialize,
{
	fn default() -> Self {
		OptionalValue::Null
	}
}

impl<T> Serialize for OptionalValue<T>
    where T: Serialize,
{
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
		where S: Serializer,
	{
		match *self {
			OptionalValue::Value(ref value) => value.serialize(serializer),
			OptionalValue::Null => Value::Null.serialize(serializer),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use util::hash::*;
	use super::*;

	#[test]
	fn test_serialize_optional_value() {
		let v: OptionalValue<H256> = OptionalValue::Null;
		let serialized = serde_json::to_string(&v).unwrap();
		assert_eq!(serialized, r#"null"#);

		let v = OptionalValue::Value(H256::default());
		let serialized = serde_json::to_string(&v).unwrap();
		assert_eq!(serialized, r#""0x0000000000000000000000000000000000000000000000000000000000000000""#);
	}
}
