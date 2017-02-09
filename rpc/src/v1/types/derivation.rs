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
