use ethabi::spec::param_type::Error as SpecError;
use ethabi::token::Error as TokenizerError;
use ethabi::Error as DecoderError;

#[derive(Debug)]
pub enum Error {
	Spec(SpecError),
	Tokenizer(TokenizerError),
	Decoder(DecoderError),
}

impl From<SpecError> for Error {
	fn from(err: SpecError) -> Self {
		Error::Spec(err)
	}
}

impl From<TokenizerError> for Error {
	fn from(err: TokenizerError) -> Self {
		Error::Tokenizer(err)
	}
}

impl From<DecoderError> for Error {
	fn from(err: DecoderError) -> Self {
		Error::Decoder(err)
	}
}
