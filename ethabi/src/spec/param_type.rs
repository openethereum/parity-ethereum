use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
	Address,
	Bytes,
	Int,
	Uint,
	Bool,
	String,
	Array(Box<ParamType>),
	FixedBytes(usize),
	FixedArray(Box<ParamType>, usize),
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
	use super::{Reader, ParamType};

	#[test]
	fn test_read_param() {
		assert_eq!(Reader::read("address").unwrap(), ParamType::Address);
		assert_eq!(Reader::read("bytes").unwrap(), ParamType::Bytes);
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
}
