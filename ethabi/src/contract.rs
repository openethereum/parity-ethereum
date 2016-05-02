use spec::Interface;
use token::Token;
use error::Error;
use coders::Encoder;

/// API building calls to contracts ABI.
pub struct Contract {
	interface: Interface,
}

impl Contract {
	/// Initializes contract with ABI specification.
	pub fn new(interface: Interface) -> Self {
		Contract {
			interface: interface
		}
	}

	/// Prepares ABI function call with given input params.
	pub fn function_call(&self, name: String, input_params: Vec<Token>) -> Result<Vec<u8>, Error> {
		//let f = try!(self.interface.function(name).ok_or(Error::FunctionNotFound));
		//let input_types = f.input_param_types();
		Ok(Encoder::encode(input_params))
	}

	/// Prepares event filter, filtering given params.
	pub fn event_filter(&self, name: String, filter_params: Vec<Token>) -> Result<(), Error> {
		let e = try!(self.interface.event(name).ok_or(Error::EventNotFound));
		let indexed_types = e.indexed_params();
		unimplemented!();
	}

	/// Parses the ABI function output to list of tokens.
	pub fn output(&self, name: String, data: Vec<u8>) -> Result<Vec<Token>, Error> {
		unimplemented!();
	}
}
