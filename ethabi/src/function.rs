//! Contract function call builder.

use spec::Function as FunctionInterface;
use token::Token;
use encoder::Encoder;
use decoder::Decoder;
use error::Error;

/// Contract function call builder.
pub struct Function {
	interface: FunctionInterface,
}

impl Function {
	/// Creates new function call builder.
	pub fn new(interface: FunctionInterface) -> Self {
		Function {
			interface: interface
		}
	}

	/// Prepares ABI function call with given input params.
	pub fn encode_call(&self, tokens: Vec<Token>) -> Result<Vec<u8>, Error> {
		// TODO: validate tokens with interface
		Ok(Encoder::encode(tokens))
	}

	/// Parses the ABI function output to list of tokens.
	pub fn decode_output(&self, data: Vec<u8>) -> Result<Vec<Token>, Error> {
		Decoder::decode(self.interface.output_param_types(), data)
	}
}

