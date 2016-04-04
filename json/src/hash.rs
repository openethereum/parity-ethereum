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

//! Lenient hash json deserialization for test json files.

use std::str::FromStr;
use serde::{Deserialize, Deserializer, Error};
use serde::de::Visitor;
use util::hash::{H64 as Hash64, Address as Hash160, H256 as Hash256, H2048 as Hash2048};


macro_rules! impl_hash {
	($name: ident, $inner: ident) => {
		/// Lenient hash json deserialization for test json files.
		#[derive(Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
		pub struct $name(pub $inner);

		impl Into<$inner> for $name {
			fn into(self) -> $inner {
				self.0
			}
		}

		impl Deserialize for $name {
			fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
				where D: Deserializer {

				struct HashVisitor;

				impl Visitor for HashVisitor {
					type Value = $name;

					fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
						let value = match value.len() {
							0 => $inner::from(0),
							2 if value == "0x" => $inner::from(0),
							_ if value.starts_with("0x") => try!($inner::from_str(&value[2..]).map_err(|_| {
								Error::custom(format!("Invalid hex value {}.", value).as_ref())
							})),
							_ => try!($inner::from_str(value).map_err(|_| {
								Error::custom(format!("Invalid hex value {}.", value).as_ref())
							}))
						};

						Ok($name(value))
					}

					fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
						self.visit_str(value.as_ref())
					}
				}

				deserializer.deserialize(HashVisitor)
			}
		}
	}
}

impl_hash!(H64, Hash64);
impl_hash!(Address, Hash160);
impl_hash!(H256, Hash256);
impl_hash!(Bloom, Hash2048);

#[cfg(test)]
mod test {
	use std::str::FromStr;
	use serde_json;
	use util::hash;
	use hash::H256;

	#[test]
	fn hash_deserialization() {
		let s = r#"["", "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae"]"#;
		let deserialized: Vec<H256> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![
				   H256(hash::H256::from(0)),
				   H256(hash::H256::from_str("5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae").unwrap())
		]);
	}

	#[test]
	fn hash_into() {
		assert_eq!(hash::H256::from(0), H256(hash::H256::from(0)).into());
	}
}
