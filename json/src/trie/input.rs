// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Trie test input deserialization.

use std::fmt;
use std::collections::BTreeMap;
use std::str::FromStr;
use bytes::Bytes;
use serde::{Deserialize, Deserializer};
use serde::de::{Error as ErrorTrait, Visitor, MapAccess, SeqAccess};

/// Trie test input.
#[derive(Debug, PartialEq)]
pub struct Input {
	/// Input params.
	pub data: BTreeMap<Bytes, Option<Bytes>>,
}

impl<'a> Deserialize<'a> for Input {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer<'a>
	{
		deserializer.deserialize_any(InputVisitor)
	}
}

struct InputVisitor;

impl<'a> Visitor<'a> for InputVisitor {
	type Value = Input;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a map of bytes into bytes")
	}

	fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error> where V: MapAccess<'a> {
		let mut result = BTreeMap::new();

		loop {
			let key_str: Option<String> = visitor.next_key()?;
			let key = match key_str {
				Some(ref k) if k.starts_with("0x") => Bytes::from_str(k).map_err(V::Error::custom)?,
				Some(k) => Bytes::new(k.into_bytes()),
				None => { break; }
			};

			let val_str: Option<String> = visitor.next_value()?;
			let val = match val_str {
				Some(ref v) if v.starts_with("0x") => Some(Bytes::from_str(v).map_err(V::Error::custom)?),
				Some(v) => Some(Bytes::new(v.into_bytes())),
				None => None,
			};

			result.insert(key, val);
		}

		let input = Input {
			data: result
		};

		Ok(input)
	}

	fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error> where V: SeqAccess<'a> {
		let mut result = BTreeMap::new();

		loop {
			let keyval: Option<Vec<Option<String>>> = visitor.next_element()?;
			let keyval = match keyval {
				Some(k) => k,
				_ => { break; },
			};

			if keyval.len() != 2 {
				return Err(V::Error::custom("Invalid key value pair."));
			}

			let ref key_str: Option<String> = keyval[0];
			let ref val_str: Option<String> = keyval[1];

			let key = match *key_str {
				Some(ref k) if k.starts_with("0x") => Bytes::from_str(k).map_err(V::Error::custom)?,
				Some(ref k) => Bytes::new(k.clone().into_bytes()),
				None => { break; }
			};

			let val = match *val_str {
				Some(ref v) if v.starts_with("0x") => Some(Bytes::from_str(v).map_err(V::Error::custom)?),
				Some(ref v) => Some(Bytes::new(v.clone().into_bytes())),
				None => None,
			};

			result.insert(key, val);
		}

		let input = Input {
			data: result
		};

		Ok(input)
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use serde_json;
	use bytes::Bytes;
	use super::Input;

	#[test]
	fn input_deserialization_from_map() {
		let s = r#"{
			"0x0045" : "0x0123456789",
			"be" : "e",
			"0x0a" : null
		}"#;

		let input: Input = serde_json::from_str(s).unwrap();
		let mut map = BTreeMap::new();
		map.insert(Bytes::new(vec![0, 0x45]), Some(Bytes::new(vec![0x01, 0x23, 0x45, 0x67, 0x89])));
		map.insert(Bytes::new(vec![0x62, 0x65]), Some(Bytes::new(vec![0x65])));
		map.insert(Bytes::new(vec![0x0a]), None);
		assert_eq!(input.data, map);
	}

	#[test]
	fn input_deserialization_from_array() {
		let s = r#"[
			["0x0045", "0x0123456789"],
			["be", "e"],
			["0x0a", null]
		]"#;

		let input: Input = serde_json::from_str(s).unwrap();
		let mut map = BTreeMap::new();
		map.insert(Bytes::new(vec![0, 0x45]), Some(Bytes::new(vec![0x01, 0x23, 0x45, 0x67, 0x89])));
		map.insert(Bytes::new(vec![0x62, 0x65]), Some(Bytes::new(vec![0x65])));
		map.insert(Bytes::new(vec![0x0a]), None);
		assert_eq!(input.data, map);
	}
}
