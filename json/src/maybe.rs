
//! Deserializer of empty string values into optionals.

use std::fmt;
use std::marker::PhantomData;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Visitor};
use serde::de::value::ValueDeserializer;

/// Deserializer of empty string values into optionals.
#[derive(Debug, PartialEq)]
pub enum MaybeEmpty<T> {
	/// Some.
	Some(T),
	/// None.
	None,
}

impl<T> Deserialize for MaybeEmpty<T> where T: Deserialize {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where D: Deserializer {
		deserializer.deserialize(MaybeEmptyVisitor::new())
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

impl<T> Visitor for MaybeEmptyVisitor<T> where T: Deserialize {
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
	use util::hash;
	use hash::H256;
	use maybe::MaybeEmpty;

	#[test]
	fn maybe_deserialization() {
		let s = r#"["", "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae"]"#;
		let deserialized: Vec<MaybeEmpty<H256>> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![
				   MaybeEmpty::None,
				   MaybeEmpty::Some(H256(hash::H256::from_str("5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae").unwrap()))
		]);
	}

}
