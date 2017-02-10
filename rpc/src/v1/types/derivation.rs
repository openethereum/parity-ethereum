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

use super::hash::{H256, H160};
use serde::{Deserialize, Deserializer, Error};
use serde::de::Visitor;
use ethstore;

pub enum DerivationType {
	Soft,
	Hard,
}

#[derive(Deserialize)]
pub struct DerivateHash {
	hash: H256,
	#[serde(rename="type")]
	d_type: DerivationType,
}

#[derive(Deserialize)]
pub struct DerivateHierarchicalItem {
	index: u64,
	#[serde(rename="type")]
	d_type: DerivationType,
}

pub type DerivateHierarchical = Vec<DerivateHierarchicalItem>;

pub enum Derivate {
	Hierarchical(DerivateHierarchical),
	Hash(DerivateHash),
}

impl From<DerivateHierarchical> for Derivate {
	fn from(d: DerivateHierarchical) -> Self {
		Derivate::Hierarchical(d)
	}
}

impl From<DerivateHash> for Derivate {
	fn from(d: DerivateHash) -> Self {
		Derivate::Hash(d)
	}
}

/// Error converting request data
#[derive(Debug)]
pub enum ConvertError {
	IndexOverlfow(u64),
}

impl Derivate {
	pub fn to_derivation(self) -> Result<ethstore::Derivation, ConvertError> {
		Ok(match self {
			Derivate::Hierarchical(drv) => {
				ethstore::Derivation::Hierarchical({
					let mut members = Vec::<ethstore::IndexDerivation>::new();
					for h in drv {
						if h.index > ::std::u32::MAX as u64 { return Err(ConvertError::IndexOverlfow(h.index)); }
						members.push(match h.d_type {
							DerivationType::Soft => ethstore::IndexDerivation { soft: true, index: h.index as u32 },
							DerivationType::Hard => ethstore::IndexDerivation { soft: false, index: h.index as u32 },
						});
					}
					members
			   })
			},
			Derivate::Hash(drv) => {
				match drv.d_type {
					DerivationType::Soft => ethstore::Derivation::SoftHash(drv.hash.into()),
					DerivationType::Hard => ethstore::Derivation::HardHash(drv.hash.into()),
				}
			},
		})
	}
}

impl Deserialize for DerivationType {
	fn deserialize<D>(deserializer: &mut D) -> Result<DerivationType, D::Error>
	where D: Deserializer {
		deserializer.deserialize(DerivationTypeVisitor)
	}
}

struct DerivationTypeVisitor;

impl Visitor for DerivationTypeVisitor {
	type Value = DerivationType;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		match value {
			"soft" => Ok(DerivationType::Soft),
			"hard" => Ok(DerivationType::Hard),
			_ => Err(Error::custom("invalid derivation type")),
		}
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}
