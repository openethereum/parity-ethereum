use std::num::ParseIntError;
use serde::{Deserialize, Deserializer, Error as SerdeError};
use serde::de::Visitor;

/// Describes function params.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
	/// Address.
	Address,
	/// Bytes.
	Bytes,
	/// Signed integer.
	Int,
	/// Unisgned integer.
	Uint,
	/// Boolean.
	Bool,
	/// String.
	String,
	/// Array of unknown size.
	Array(Box<ParamType>),
	/// Vector of bytes with fixed size.
	FixedBytes(usize),
	/// Array with fixed size.
	FixedArray(Box<ParamType>, usize),
}

impl Deserialize for ParamType {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
		deserializer.deserialize(ParamTypeVisitor)
	}
}

struct ParamTypeVisitor;

impl Visitor for ParamTypeVisitor {
	type Value = ParamType;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		Reader::read(value).map_err(|e| SerdeError::custom(format!("{:?}", e).as_ref()))
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_ref())
	}
}

#[derive(Debug)]
pub enum Error {
	InvalidType,
	ParseInt(ParseIntError),
}

impl From<ParseIntError> for Error {
	fn from(err: ParseIntError) -> Self {
		Error::ParseInt(err)
	}
}

pub struct Reader;

impl Reader {
	pub fn read(name: &str) -> Result<ParamType, Error> {
		// check if it is a fixed or dynamic array.
		if let Some(']') = name.chars().last() {
			// take number part
			let num: String = name.chars()
				.rev()
				.skip(1)
				.take_while(|c| *c != '[')
				.collect::<String>()
				.chars()
				.rev()
				.collect();

			let count = name.chars().count();
			if num.len() == 0 {
				// we already know it's a dynamic array!
				let subtype = try!(Reader::read(&name[..count - 2]));
				return Ok(ParamType::Array(Box::new(subtype)));
			} else {
				// it's a fixed array.
				let len = try!(usize::from_str_radix(&num, 10));
				let subtype = try!(Reader::read(&name[..count - num.len() - 2]));
				return Ok(ParamType::FixedArray(Box::new(subtype), len));
			}
		}

		let result = match name {
			"address" => ParamType::Address,
			"bytes" => ParamType::Bytes,
			"bool" => ParamType::Bool,
			"string" => ParamType::String,
			"int" => ParamType::Int,
			"uint" => ParamType::Uint,
			s if s.starts_with("int") => {
				let _ = try!(usize::from_str_radix(&s[5..], 10));
				ParamType::Int
			},
			s if s.starts_with("uint") => {
				let _ = try!(usize::from_str_radix(&s[5..], 10));
				ParamType::Uint
			},
			s if s.starts_with("bytes") => {
				let len = try!(usize::from_str_radix(&s[5..], 10));
				ParamType::FixedBytes(len)
			},
			_ => {
				return Err(Error::InvalidType);
			}
		};

		Ok(result)
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::{Reader, ParamType};

	#[test]
	fn test_read_param() {
		assert_eq!(Reader::read("address").unwrap(), ParamType::Address);
		assert_eq!(Reader::read("bytes").unwrap(), ParamType::Bytes);
		assert_eq!(Reader::read("bytes32").unwrap(), ParamType::FixedBytes(32));
		assert_eq!(Reader::read("bool").unwrap(), ParamType::Bool);
		assert_eq!(Reader::read("string").unwrap(), ParamType::String);
		assert_eq!(Reader::read("int").unwrap(), ParamType::Int);
		assert_eq!(Reader::read("uint").unwrap(), ParamType::Uint);
	}

	#[test]
	fn test_read_array_param() {
		assert_eq!(Reader::read("address[]").unwrap(), ParamType::Array(Box::new(ParamType::Address)));
		assert_eq!(Reader::read("uint[]").unwrap(), ParamType::Array(Box::new(ParamType::Uint)));
		assert_eq!(Reader::read("bytes[]").unwrap(), ParamType::Array(Box::new(ParamType::Bytes)));
		assert_eq!(Reader::read("bool[][]").unwrap(), ParamType::Array(Box::new(ParamType::Array(Box::new(ParamType::Bool)))));
	}

	#[test]
	fn test_read_fixed_array_param() {
		assert_eq!(Reader::read("address[2]").unwrap(), ParamType::FixedArray(Box::new(ParamType::Address), 2));
		assert_eq!(Reader::read("bool[17]").unwrap(), ParamType::FixedArray(Box::new(ParamType::Bool), 17));
		assert_eq!(Reader::read("bytes[45][3]").unwrap(), ParamType::FixedArray(Box::new(ParamType::FixedArray(Box::new(ParamType::Bytes), 45)), 3));
	}

	#[test]
	fn test_read_mixed_arrays() {
		assert_eq!(Reader::read("bool[][3]").unwrap(), ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 3));
		assert_eq!(Reader::read("bool[3][]").unwrap(), ParamType::Array(Box::new(ParamType::FixedArray(Box::new(ParamType::Bool), 3))));
	}
	
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
			ParamType::Int,
			ParamType::Uint,
			ParamType::Array(Box::new(ParamType::Address)),
			ParamType::FixedArray(Box::new(ParamType::Uint), 3),
			ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 5)
		]);
	}

}
