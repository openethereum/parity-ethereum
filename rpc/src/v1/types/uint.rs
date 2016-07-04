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

use std::cmp;
use std::str::FromStr;
use rustc_serialize::hex::ToHex;
use serde;
use util::{U256 as EthU256, Uint};

macro_rules! impl_uint {
	($name: ident, $other: ident, $size: expr) => {
		/// Uint serialization.
		#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
		pub struct $name($other);

		impl<T> From<T> for $name where $other: From<T> {
			fn from(o: T) -> Self {
				$name($other::from(o))
			}
		}

		impl FromStr for $name {
			type Err = <$other as FromStr>::Err;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				$other::from_str(s).map($name)
			}
		}

		impl Into<$other> for $name {
			fn into(self) -> $other {
				self.0
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: serde::Serializer {
				let mut hex = "0x".to_owned();
				let mut bytes = [0u8; 8 * $size];
				self.0.to_raw_bytes(&mut bytes);
				let len = cmp::max((self.0.bits() + 7) / 8, 1);
				hex.push_str(&bytes[bytes.len() - len..].to_hex());
				serializer.serialize_str(&hex)
			}
		}

		impl serde::Deserialize for $name {
			fn deserialize<D>(deserializer: &mut D) -> Result<$name, D::Error>
			where D: serde::Deserializer {
				struct UintVisitor;

				impl serde::de::Visitor for UintVisitor {
					type Value = $name;

					fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: serde::Error {
						// 0x + len
						if value.len() > 2 + $size * 16 || value.len() < 2 {
							return Err(serde::Error::custom("Invalid length."));
						}

						$other::from_str(&value[2..]).map($name).map_err(|_| serde::Error::custom("Invalid hex value."))
					}

					fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: serde::Error {
						self.visit_str(&value)
					}
				}

				deserializer.deserialize(UintVisitor)
			}
		}

	}
}

impl_uint!(U256, EthU256, 4);
