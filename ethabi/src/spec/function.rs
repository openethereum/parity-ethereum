//! Contract function specification.

use super::{Param, ParamType};

/// Contract function specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Function {
	/// Function name.
	pub name: String,
	/// Function input.
	pub inputs: Vec<Param>,
	/// Function output.
	pub outputs: Vec<Param>,
}

impl Function {
	/// Returns all input params of given function.
	pub fn input_param_types(&self) -> Vec<ParamType> {
		self.inputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}

	/// Returns all output params of given function.
	pub fn output_param_types(&self) -> Vec<ParamType> {
		self.outputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}
}
