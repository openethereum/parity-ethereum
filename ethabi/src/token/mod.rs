//! ABI param and parsing for it.

mod error;
mod lenient;
mod strict;
mod token;

use spec::ParamType;
pub use self::error::Error;
pub use self::lenient::LenientTokenizer;
pub use self::strict::StrictTokenizer;
pub use self::token::Token;

/// This trait should be used to parse string values as tokens.
pub trait Tokenizer {
	/// Tries to parse a string as a token of given type.
	fn tokenize(param: &ParamType, value: &str) -> Result<Token, Error> {
		match *param {
			ParamType::Address => Self::tokenize_address(value).map(Token::Address),
			ParamType::String => Self::tokenize_string(value).map(Token::String),
			ParamType::Bool => Self::tokenize_bool(value).map(Token::Bool),
			ParamType::Bytes => Self::tokenize_bytes(value).map(Token::Bytes),
			ParamType::FixedBytes(len) => Self::tokenize_fixed_bytes(value, len).map(Token::FixedBytes),
			ParamType::Uint(_) => Self::tokenize_uint(value).map(Token::Uint),
			ParamType::Int(_) => Self::tokenize_int(value).map(Token::Int),
			_ => {
				unimplemented!();
			}
		}
	}

	/// Tries to parse a value as an address.
	fn tokenize_address(value: &str) -> Result<[u8; 20], Error>;

	/// Tries to parse a value as a string.
	fn tokenize_string(value: &str) -> Result<String, Error>;

	/// Tries to parse a value as a bool.
	fn tokenize_bool(value: &str) -> Result<bool, Error>;

	/// Tries to parse a value as bytes.
	fn tokenize_bytes(value: &str) -> Result<Vec<u8>, Error>;

	/// Tries to parse a value as bytes.
	fn tokenize_fixed_bytes(value: &str, len: usize) -> Result<Vec<u8>, Error>;

	/// Tries to parse a value as unsigned integer.
	fn tokenize_uint(value: &str) -> Result<[u8; 32], Error>;

	/// Tries to parse a value as signed integer.
	fn tokenize_int(value: &str) -> Result<[u8; 32], Error>;
}
