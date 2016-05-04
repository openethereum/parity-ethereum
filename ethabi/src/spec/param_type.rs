#[derive(Debug, Clone)]
pub enum ParamType {
	Address,
	Bytes,
	Int,
	Uint,
	Bool,
	String,
	Array(Vec<ParamType>),
	FixedBytes(usize),
	FixedArray(Vec<ParamType>),
}
