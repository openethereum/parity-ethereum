//! Ethereum ABI params.
use std::fmt::{Display, Formatter, Error};
use rustc_serialize::hex::ToHex;

/// Ethereum ABI params.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	/// Address.
	/// 
	/// solidity name: address
	/// Encoded to left padded [0u8; 32].
	Address([u8;20]),
	/// Vector of bytes with known size.
	/// 
	/// solidity name eg.: bytes8, bytes32, bytes64, bytes1024
	/// Encoded to right padded [0u8; ((N + 31) / 32) * 32].
	FixedBytes(Vec<u8>),
	/// Vector of bytes of unknown size.
	/// 
	/// solidity name: bytes
	/// Encoded in two parts.
	/// Init part: offset of 'closing part`.
	/// Closing part: encoded length followed by encoded right padded bytes.
	Bytes(Vec<u8>),
	/// Signed integer.
	/// 
	/// solidity name: int
	Int([u8;32]),
	/// Unisnged integer.
	/// 
	/// solidity name: uint
	Uint([u8;32]),
	/// Boolean value.
	/// 
	/// solidity name: bool
	/// Encoded as left padded [0u8; 32], where last bit represents boolean value.
	Bool(bool),
	/// String.
	/// 
	/// solidity name: string
	/// Encoded in the same way as bytes. Must be utf8 compliant.
	String(String),
	/// Array with known size.
	/// 
	/// solidity name eg.: int[3], bool[3], address[][8]
	/// Encoding of array is equal to encoding of consecutive elements of array.
	FixedArray(Vec<Token>),
	/// Array of params with unknown size.
	/// 
	/// solidity name eg. int[], bool[], address[5][]
	Array(Vec<Token>),
}

impl Display for Token {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		match *self {
			Token::Bool(b) => write!(f, "{}", b),
			Token::String(ref s) => write!(f, "{}", s),
			Token::Address(ref a) => write!(f, "{}", a.to_hex()),
			Token::Bytes(ref bytes) | Token::FixedBytes(ref bytes) => write!(f, "{}", bytes.to_hex()),
			Token::Uint(ref i) | Token::Int(ref i) => write!(f, "{}", i.to_hex()),
			Token::Array(ref arr) | Token::FixedArray(ref arr) => {
				let s = arr.iter()
					.map(|ref t| format!("{}", t))
					.collect::<Vec<String>>()
					.join(",");

				write!(f, "[{}]", s)
			}
		}
	}
}

impl Token {
	/// Converts token to...
	pub fn to_address(self) -> Option<[u8; 20]> {
		match self {
			Token::Address(address) => Some(address),
			_ => None,
		}
	}

	/// Converts token to...
	pub fn to_fixed_bytes(self) -> Option<Vec<u8>> {
		match self {
			Token::FixedBytes(bytes) => Some(bytes),
			_ => None,
		}
	}

	/// Converts token to...
	pub fn to_bytes(self) -> Option<Vec<u8>> {
		match self {
			Token::Bytes(bytes) => Some(bytes),
			_ => None,
		}
	}

	/// Converts token to...
	pub fn to_int(self) -> Option<[u8; 32]> {
		match self {
			Token::Int(int) => Some(int),
			_ => None,
		}
	}

	/// Converts token to...
	pub fn to_uint(self) -> Option<[u8; 32]> {
		match self {
			Token::Uint(uint) => Some(uint),
			_ => None,
		}
	}

	/// Converts token to...
	pub fn to_bool(self) -> Option<bool> {
		match self {
			Token::Bool(b) => Some(b),
			_ => None,
		}
	}

	/// Converts token to...
	pub fn to_string(self) -> Option<String> {
		match self {
			Token::String(s) => Some(s),
			_ => None,
		}
	}

	/// Converts token to...
	pub fn to_fixed_array(self) -> Option<Vec<Token>> {
		match self {
			Token::FixedArray(arr) => Some(arr),
			_ => None,
		}
	}

	/// Converts token to...
	pub fn to_array(self) -> Option<Vec<Token>> {
		match self {
			Token::Array(arr) => Some(arr),
			_ => None,
		}
	}
}
