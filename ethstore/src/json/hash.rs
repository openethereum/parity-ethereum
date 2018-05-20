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

use std::{ops, fmt, str};
use rustc_hex::{FromHex, ToHex};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor, Error as SerdeError};
use super::Error;

macro_rules! impl_hash {
	($name: ident, $size: expr) => {
		pub struct $name([u8; $size]);

		impl fmt::Debug for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
				let self_ref: &[u8] = &self.0;
				write!(f, "{:?}", self_ref)
			}
		}

		impl PartialEq for $name {
			fn eq(&self, other: &Self) -> bool {
				let self_ref: &[u8] = &self.0;
				let other_ref: &[u8] = &other.0;
				self_ref == other_ref
			}
		}

		impl ops::Deref for $name {
			type Target = [u8];

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where S: Serializer {
				serializer.serialize_str(&self.0.to_hex())
			}
		}

		impl<'a> Deserialize<'a> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where D: Deserializer<'a> {
				struct HashVisitor;

				impl<'b> Visitor<'b> for HashVisitor {
					type Value = $name;

					fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
						write!(formatter, "a hex-encoded {}", stringify!($name))
					}

					fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
						value.parse().map_err(SerdeError::custom)
					}

					fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
						self.visit_str(value.as_ref())
					}
				}

				deserializer.deserialize_any(HashVisitor)
			}
		}

		impl str::FromStr for $name {
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

		impl From<&'static str> for $name {
			fn from(s: &'static str) -> Self {
				s.parse().expect(&format!("invalid string literal for {}: '{}'", stringify!($name), s))
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
