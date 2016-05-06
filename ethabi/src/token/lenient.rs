use token::Tokenizer;

pub struct LenientTokenizer;

impl Tokenizer for LenientTokenizer {
	fn tokenize_address(_value: &str) -> Option<[u8; 20]> {
		unimplemented!();
	}

	fn tokenize_string(_value: &str) -> Option<String> {
		unimplemented!();
	}

	fn tokenize_bool(_value: &str) -> Option<bool> {
		unimplemented!();
	}
}
