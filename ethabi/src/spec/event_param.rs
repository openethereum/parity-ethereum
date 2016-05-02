use super::ParamType;

#[derive(Debug, Clone)]
pub struct EventParam {
	pub name: String,
	pub kind: ParamType,
	pub indexed: bool,
}
