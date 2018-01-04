// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::fmt;
use std::str::FromStr;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use serde;
use rustc_hex::{ToHex, FromHex};
use ethereum_types::{H64 as Eth64, H160 as Eth160, H256 as Eth256, H520 as Eth520, H512 as Eth512, H2048 as Eth2048};

macro_rules! impl_hash {
	($name: ident, $other: ident, $size: expr) => {
		/// Hash serialization
		pub struct $name(pub [u8; $size]);

		impl Eq for $name { }

		impl Default for $name {
			fn default() -> Self {
				$name([0; $size])
			}
		}

		impl fmt::Debug for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "{}", self.0.to_hex())
			}
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				let hex = self.0.to_hex();
				write!(f, "{}..{}", &hex[0..2], &hex[$size-2..$size])
			}
		}

		impl<T> From<T> for $name where $other: From<T> {
			fn from(o: T) -> Self {
				$name($other::from(o).0)
			}
		}

		impl FromStr for $name {
			type Err = <$other as FromStr>::Err;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				$other::from_str(s).map(|x| $name(x.0))
			}
		}

		impl Into<$other> for $name {
			fn into(self) -> $other {
				$other(self.0)
			}
		}

		impl PartialEq for $name {
			fn eq(&self, other: &Self) -> bool {
				let self_ref: &[u8] = &self.0;
				let other_ref: &[u8] = &other.0;
				self_ref == other_ref
			}
		}

		impl PartialOrd for $name {
			fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
				let self_ref: &[u8] = &self.0;
				let other_ref: &[u8] = &other.0;
				self_ref.partial_cmp(other_ref)
			}
		}

		impl Ord for $name {
			fn cmp(&self, other: &Self) -> Ordering {
				let self_ref: &[u8] = &self.0;
				let other_ref: &[u8] = &other.0;
				self_ref.cmp(other_ref)
			}
		}

		impl Hash for $name {
			fn hash<H>(&self, state: &mut H) where H: Hasher {
				let self_ref: &[u8] = &self.0;
				Hash::hash(self_ref, state)
			}
		}

		impl Clone for $name {
			fn clone(&self) -> Self {
				let mut r = [0; $size];
				r.copy_from_slice(&self.0);
				$name(r)
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where S: serde::Serializer {
				let mut hex = "0x".to_owned();
				hex.push_str(&self.0.to_hex());
				serializer.serialize_str(&hex)
			}
		}

		impl<'a> serde::Deserialize<'a> for $name {
			fn deserialize<D>(deserializer: D) -> Result<$name, D::Error> where D: serde::Deserializer<'a> {
				struct HashVisitor;

				impl<'b> serde::de::Visitor<'b> for HashVisitor {
					type Value = $name;

					fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
						write!(formatter, "a 0x-prefixed, padded, hex-encoded hash with length {}", $size * 2)
					}

					fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: serde::de::Error {

						if value.len() < 2 || &value[0..2] != "0x" {
							return Err(E::custom("expected a hex-encoded hash with 0x prefix"));
						}
						if value.len() != 2 + $size * 2 {
							return Err(E::invalid_length(value.len() - 2, &self));
						}

						match value[2..].from_hex() {
							Ok(ref v) => {
								let mut result = [0u8; $size];
								result.copy_from_slice(v);
								Ok($name(result))
							},
							Err(e) => Err(E::custom(format!("invalid hex value: {:?}", e))),
						}
					}

					fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: serde::de::Error {
						self.visit_str(value.as_ref())
					}
				}

				deserializer.deserialize_any(HashVisitor)
			}
		}
	}
}

impl_hash!(H64, Eth64, 8);
impl_hash!(H160, Eth160, 20);
impl_hash!(H256, Eth256, 32);
impl_hash!(H512, Eth512, 64);
impl_hash!(H520, Eth520, 65);
impl_hash!(H2048, Eth2048, 256);
