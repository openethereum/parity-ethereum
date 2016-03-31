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

//! Lenient uint json deserialization for test json files.

use std::str::FromStr;
use serde::{Deserialize, Deserializer, Error};
use serde::de::Visitor;
use util::numbers::{U256, Uint as U};

/// Lenient uint json deserialization for test json files.
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Uint(pub U256);

impl Into<U256> for Uint {
	fn into(self) -> U256 {
		self.0
	}
}

impl Into<u64> for Uint {
	fn into(self) -> u64 {
		u64::from(self.0)
	}
}

impl Into<usize> for Uint {
	fn into(self) -> usize {
		// TODO: clean it after util conversions refactored.
		u64::from(self.0) as usize
	}
}
impl Into<u8> for Uint {
	fn into(self) -> u8 {
		u64::from(self.0) as u8
	}
}

impl Deserialize for Uint {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
		where D: Deserializer {
		deserializer.deserialize(UintVisitor)
	}
}

struct UintVisitor;

impl Visitor for UintVisitor {
	type Value = Uint;

	fn visit_u64<E>(&mut self, value: u64) -> Result<Self::Value, E> where E: Error {
		Ok(Uint(U256::from(value)))
	}

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		let value = match value.len() {
			0 => U256::from(0),
			2 if value.starts_with("0x") => U256::from(0),
			_ if value.starts_with("0x") => try!(U256::from_str(&value[2..]).map_err(|_| {
				Error::custom(format!("Invalid hex value {}.", value).as_ref())
			})),
			_ => try!(U256::from_dec_str(value).map_err(|_| {
				Error::custom(format!("Invalid decimal value {}.", value).as_ref())
			}))
		};

		Ok(Uint(value))
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

#[cfg(test)]
mod test {
	use serde_json;
	use util::numbers::U256;
	use uint::Uint;

	#[test]
	fn uint_deserialization() {
		let s = r#"["0xa", "10", "", "0x", 0]"#;
		let deserialized: Vec<Uint> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![
				   Uint(U256::from(10)),
				   Uint(U256::from(10)),
				   Uint(U256::from(0)),
				   Uint(U256::from(0)),
				   Uint(U256::from(0))
		]);
	}

	#[test]
	fn uint_into() {
		assert_eq!(U256::from(10), Uint(U256::from(10)).into());
	}
}
