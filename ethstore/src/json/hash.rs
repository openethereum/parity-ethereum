use std::str::FromStr;
use rustc_serialize::hex::{FromHex, ToHex};
use serde::{Serialize, Serializer, Deserialize, Deserializer, Error as SerdeError};
use serde::de::Visitor;
use super::Error;

macro_rules! impl_hash {
	($name: ident, $size: expr) => {
		#[derive(Debug, PartialEq)]
		pub struct $name([u8; $size]);

		impl Serialize for $name {
			fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
			where S: Serializer {
				serializer.serialize_str(&self.0.to_hex())
			}
		}

		impl Deserialize for $name {
			fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
			where D: Deserializer {
				struct HashVisitor;

				impl Visitor for HashVisitor {
					type Value = $name;

					fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
						FromStr::from_str(value).map_err(SerdeError::custom)
					}

					fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: SerdeError {
						self.visit_str(value.as_ref())
					}
				}

				deserializer.deserialize(HashVisitor)
			}
		}

		impl FromStr for $name {
			type Err = Error;

			fn from_str(value: &str) -> Result<Self, Self::Err> {
				match value.from_hex() {
					Ok(ref hex) if hex.len() == $size => {
						let mut hash = [0u8; $size];
						hash.clone_from_slice(hex);
						Ok($name(hash))
					}
					_ => Err(Error::InvalidH256),
				}
			}
		}

		impl From<[u8; $size]> for $name {
			fn from(bytes: [u8; $size]) -> Self {
				$name(bytes)
			}
		}

		impl Into<[u8; $size]> for $name {
			fn into(self) -> [u8; $size] {
				self.0
			}
		}
	}
}

impl_hash!(H128, 16);
impl_hash!(H160, 20);
impl_hash!(H256, 32);
