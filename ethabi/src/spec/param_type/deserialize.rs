use std::fmt;
use serde::{Deserialize, Deserializer};
use serde::de::{Error as SerdeError, Visitor};
use super::{ParamType, Reader};

impl Deserialize for ParamType {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer {
		deserializer.deserialize(ParamTypeVisitor)
	}
}

struct ParamTypeVisitor;

impl Visitor for ParamTypeVisitor {
	type Value = ParamType;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a correct name of abi-encodable parameter type")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		Reader::read(value).map_err(|e| SerdeError::custom(format!("{:?}", e).as_str()))
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_str())
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::ParamType;

	#[test]
	fn param_type_deserialization() {
		let s = r#"["address", "bytes", "bytes32", "bool", "string", "int", "uint", "address[]", "uint[3]", "bool[][5]"]"#;
		let deserialized: Vec<ParamType> = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized, vec![
			ParamType::Address,
			ParamType::Bytes,
			ParamType::FixedBytes(32),
			ParamType::Bool,
			ParamType::String,
			ParamType::Int(256),
			ParamType::Uint(256),
			ParamType::Array(Box::new(ParamType::Address)),
			ParamType::FixedArray(Box::new(ParamType::Uint(256)), 3),
			ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 5)
		]);
	}
}
