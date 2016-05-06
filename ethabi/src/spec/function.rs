use super::{Param, ParamType};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Function {
	pub name: String,
	pub inputs: Vec<Param>,
	pub outputs: Vec<Param>,
}

impl Function {
	pub fn input_param_types(&self) -> Vec<ParamType> {
		self.inputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}

	pub fn output_param_types(&self) -> Vec<ParamType> {
		self.outputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}
}
