//! jsonrpc id field
use serde::{Serialize, Serializer, Deserialize, Deserializer, Error};
use serde::de::Visitor;

#[derive(Debug, PartialEq)]
pub enum Id {
	Null,
	Num(u64),
}

impl Serialize for Id {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> 
	where S: Serializer {
		match self {
			&Id::Null => serializer.visit_unit(),
			&Id::Num(v) => serializer.visit_u64(v)
		}
	}
}

impl Deserialize for Id {
	fn deserialize<D>(deserializer: &mut D) -> Result<Id, D::Error>
	where D: Deserializer {
		deserializer.visit(IdVisitor)
	}
}

struct IdVisitor;

impl Visitor for IdVisitor {
	type Value = Id;

	fn visit_unit<E>(&mut self) -> Result<Self::Value, E> where E: Error {
		Ok(Id::Null)
	}

	fn visit_u64<E>(&mut self, value: u64) -> Result<Self::Value, E> where E: Error {
		Ok(Id::Num(value))
	}

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		self.visit_string(value.to_string())
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		value.parse::<u64>().map(Id::Num).map_err(|_| Error::syntax("invalid id"))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;

	#[test]
	fn id_deserialization() {
		let s = r#"[null, 0, 2, "3"]"#;
		let deserialized: Vec<Id> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![Id::Null, Id::Num(0), Id::Num(2), Id::Num(3)]);
	}

	#[test]
	fn id_serialization() {
		let d = vec![Id::Null, Id::Num(0), Id::Num(2), Id::Num(3)];
		let serialized = serde_json::to_string(&d).unwrap();
		assert_eq!(serialized, r#"[null,0,2,3]"#);
	}
}
