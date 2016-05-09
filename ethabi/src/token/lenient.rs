use token::{Tokenizer, Error};

/// Tries to parse string as a token. Does not require string to clearly represent the value.
pub struct LenientTokenizer;

impl Tokenizer for LenientTokenizer {
	fn tokenize_address(_value: &str) -> Result<[u8; 20], Error> {
		unimplemented!();
	}

	fn tokenize_string(_value: &str) -> Result<String, Error> {
		unimplemented!();
	}

	fn tokenize_bool(_value: &str) -> Result<bool, Error> {
		unimplemented!();
	}

	fn tokenize_bytes(_value: &str) -> Result<Vec<u8>, Error> {
		unimplemented!();
	}

	fn tokenize_fixed_bytes(_value: &str, _len: usize) -> Result<Vec<u8>, Error> {
		unimplemented!();
	}

	fn tokenize_uint(_value: &str) -> Result<[u8; 32], Error> {
		unimplemented!();
	}

	fn tokenize_int(_value: &str) -> Result<[u8; 32], Error> {
		unimplemented!();
	}
}
