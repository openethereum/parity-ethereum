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

//! Deserializer of empty string values into optionals.

use std::fmt;
use std::marker::PhantomData;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Visitor, IntoDeserializer};

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
		where D: Deserializer<'a> {
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
		match value.is_empty() {
			true => Ok(MaybeEmpty::None),
			false => {
				T::deserialize(value.into_deserializer()).map(MaybeEmpty::Some)
			}
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
mod tests {
	use std::str::FromStr;
	use serde_json;
	use ethereum_types;
	use hash::H256;
	use maybe::MaybeEmpty;

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
