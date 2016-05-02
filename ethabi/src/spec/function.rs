use super::{Param, ParamType};

#[derive(Debug, Clone)]
pub struct Function {
	pub name: String,
	pub inputs: Vec<Param>,
	pub outputs: Option<Vec<Param>>,
}

impl Function {
	pub fn input_param_types(&self) -> Vec<ParamType> {
		self.inputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}

	pub fn output_param_types(&self) -> Option<Vec<ParamType>> {
		self.outputs.as_ref().map(|o| {
			o.iter()
				.map(|p| p.kind.clone())
				.collect()
		})
	}
}
