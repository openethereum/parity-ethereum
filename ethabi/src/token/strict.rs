use std::ptr;
use rustc_serialize::hex::FromHex;
use token::Tokenizer;

pub struct StrictTokenizer;

impl Tokenizer for StrictTokenizer {
	fn tokenize_address(value: &str) -> Option<[u8; 20]> {
		let hex = value.from_hex().expect("TODO!");
		match hex.len() == 20 {
			false => None,
			true => {
				let mut address = [0u8; 20];
				unsafe {
					ptr::copy(hex.as_ptr(), address.as_mut_ptr(), 20);
				}
				Some(address)
			}
		}
	}

	fn tokenize_string(value: &str) -> Option<String> {
		Some(value.to_owned())
	}

	fn tokenize_bool(value: &str) -> Option<bool> {
		match value {
			"true" | "1" => Some(true),
			"false" | "0" => Some(false),
			_ => None
		}
	}
}

#[cfg(test)]
mod tests {
	use spec::ParamType;
	use token::{Token, Tokenizer, StrictTokenizer};

	#[test]
	fn tokenize_address() {
		assert_eq!(StrictTokenizer::tokenize(&ParamType::Address, "1111111111111111111111111111111111111111"), Some(Token::Address([0x11u8; 20])));
		assert_eq!(StrictTokenizer::tokenize(&ParamType::Address, "2222222222222222222222222222222222222222"), Some(Token::Address([0x22u8; 20])));
	}

	#[test]
	fn tokenize_string() {
		assert_eq!(StrictTokenizer::tokenize(&ParamType::String, "gavofyork"), Some(Token::String("gavofyork".to_owned())));
		assert_eq!(StrictTokenizer::tokenize(&ParamType::String, "hello"), Some(Token::String("hello".to_owned())));
	}

	#[test]
	fn tokenize_bool() {
		assert_eq!(StrictTokenizer::tokenize(&ParamType::Bool, "true"), Some(Token::Bool(true)));
		assert_eq!(StrictTokenizer::tokenize(&ParamType::Bool, "1"), Some(Token::Bool(true)));
		assert_eq!(StrictTokenizer::tokenize(&ParamType::Bool, "false"), Some(Token::Bool(false)));
		assert_eq!(StrictTokenizer::tokenize(&ParamType::Bool, "false"), Some(Token::Bool(false)));
	}
}
