mod token;
mod lenient;
mod strict;

use spec::ParamType;
pub use self::token::Token;
pub use self::lenient::LenientTokenizer;
pub use self::strict::StrictTokenizer;

pub trait Tokenizer {
	fn tokenize(param: &ParamType, value: &str) -> Option<Token> {
		match *param {
			ParamType::Address => Self::tokenize_address(value).map(Token::Address),
			ParamType::String => Self::tokenize_string(value).map(Token::String),
			ParamType::Bool => Self::tokenize_bool(value).map(Token::Bool),
			_ => {
				unimplemented!();
			}
		}
	}

	fn tokenize_address(value: &str) -> Option<[u8; 20]>;

	fn tokenize_string(value: &str) -> Option<String>;

	fn tokenize_bool(value: &str) -> Option<bool>;
}
