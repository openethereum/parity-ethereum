use spec::ParamType;
use error::Error;
use token::Token;

pub struct Decoder;

impl Decoder {
	pub fn decode(types: Vec<ParamType>, data: Vec<u8>) -> Result<Vec<Token>, Error> {
		unimplemented!();
	}
}

