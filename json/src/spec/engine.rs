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

//! Engine deserialization.

use serde::{Deserialize, Deserializer, Error};
use serde::de::Visitor;

/// Engine deserialization.
#[derive(Debug, PartialEq)]
pub enum Engine {
	/// Null engine.
	Null,
	/// Ethash engine.
	Ethash,
}

impl Deserialize for Engine {
	fn deserialize<D>(deserializer: &mut D) -> Result<Engine, D::Error>
	where D: Deserializer {
		deserializer.deserialize(EngineVisitor)
	}
}

struct EngineVisitor;

impl Visitor for EngineVisitor {
	type Value = Engine;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		match value {
			"NullEngine" => Ok(Engine::Null),
			"Ethash" => Ok(Engine::Ethash),
			_ => Err(Error::custom("invalid engine"))
		}
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::Engine;

	#[test]
	fn engine_deserialization() {
		let s = r#"["NullEngine", "Ethash"]"#;
		let deserialized: Vec<Engine> = serde_json::from_str(s).unwrap();
		assert_eq!(vec![Engine::Null, Engine::Ethash], deserialized);
	}

	#[test]
	fn invalid_engine_deserialization() {
		let s = r#"["Etash"]"#;
		let deserialized: Result<Vec<Engine>, _> = serde_json::from_str(s);
		assert!(deserialized.is_err());
	}
}

