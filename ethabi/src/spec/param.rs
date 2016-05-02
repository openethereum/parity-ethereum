use super::ParamType;

#[derive(Debug, Clone)]
pub struct Param {
	pub name: String,
	pub kind: ParamType,
}
