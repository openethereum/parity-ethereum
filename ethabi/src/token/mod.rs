mod error;
mod lenient;
mod strict;
mod token;

use spec::ParamType;
pub use self::error::Error;
pub use self::lenient::LenientTokenizer;
pub use self::strict::StrictTokenizer;
pub use self::token::Token;

pub trait Tokenizer {
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

	fn tokenize_address(value: &str) -> Result<[u8; 20], Error>;

	fn tokenize_string(value: &str) -> Result<String, Error>;

	fn tokenize_bool(value: &str) -> Result<bool, Error>;

	fn tokenize_bytes(value: &str) -> Result<Vec<u8>, Error>;

	fn tokenize_fixed_bytes(value: &str, len: usize) -> Result<Vec<u8>, Error>;

	fn tokenize_uint(value: &str) -> Result<[u8; 32], Error>;

	fn tokenize_int(value: &str) -> Result<[u8; 32], Error>;
}
