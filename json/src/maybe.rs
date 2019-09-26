// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Deserializer of empty string values into optionals.

use std::fmt;
use std::marker::PhantomData;

use ethereum_types::U256;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Visitor, IntoDeserializer};

use crate::uint::Uint;

/// Deserializer of empty string values into optionals.
#[derive(Debug, PartialEq, Clone)]
pub enum MaybeEmpty<T> {
	/// Some.
	Some(T),
	/// None.
	None,
}

impl<'a, T> Deserialize<'a> for MaybeEmpty<T> where T: Deserialize<'a> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer<'a>
	{
		deserializer.deserialize_any(MaybeEmptyVisitor::new())
	}
}

struct MaybeEmptyVisitor<T> {
	_phantom: PhantomData<T>
}

impl<T> MaybeEmptyVisitor<T> {
	fn new() -> Self {
		MaybeEmptyVisitor {
			_phantom: PhantomData
		}
	}
}

impl<'a, T> Visitor<'a> for MaybeEmptyVisitor<T> where T: Deserialize<'a> {
	type Value = MaybeEmpty<T>;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "an empty string or string-encoded type")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: Error {
		self.visit_string(value.to_owned())
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: Error {
		if value.is_empty() {
			Ok(MaybeEmpty::None)
		} else {
			T::deserialize(value.into_deserializer()).map(MaybeEmpty::Some)
		}
	}
}

impl<T> Into<Option<T>> for MaybeEmpty<T> {
	fn into(self) -> Option<T> {
		match self {
			MaybeEmpty::Some(s) => Some(s),
			MaybeEmpty::None => None
		}
	}
}

#[cfg(test)]
impl From<Uint> for MaybeEmpty<Uint> {
	fn from(uint: Uint) -> Self {
		MaybeEmpty::Some(uint)
	}
}

impl From<MaybeEmpty<Uint>> for U256 {
	fn from(maybe: MaybeEmpty<Uint>) -> U256 {
		match maybe {
			MaybeEmpty::Some(v) => v.0,
			MaybeEmpty::None => U256::zero(),
		}
	}
}

impl From<MaybeEmpty<Uint>> for u64 {
	fn from(maybe: MaybeEmpty<Uint>) -> u64 {
		match maybe {
			MaybeEmpty::Some(v) => v.0.low_u64(),
			MaybeEmpty::None => 0u64,
		}
	}
}

impl Default for MaybeEmpty<Uint> {
	fn default() -> Self {
		MaybeEmpty::Some(Uint::default())
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use super::MaybeEmpty;
	use crate::hash::H256;

	#[test]
	fn maybe_deserialization() {
		let s = r#"["", "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae"]"#;
		let deserialized: Vec<MaybeEmpty<H256>> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![
				   MaybeEmpty::None,
				   MaybeEmpty::Some(H256(ethereum_types::H256::from_str("5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae").unwrap()))
		]);
	}

}
