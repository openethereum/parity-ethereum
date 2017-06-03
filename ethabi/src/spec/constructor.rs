//! Contract constructor specification.

use super::{Param, ParamType};

/// Contract constructor specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Constructor {
	/// Constructor input.
	pub inputs: Vec<Param>,
}

impl Constructor {
	/// Returns all input params of given constructor.
	pub fn param_types(&self) -> Vec<ParamType> {
		self.inputs.iter()
			.map(|p| p.kind.clone())
			.collect()
	}
}
