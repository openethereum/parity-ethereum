#[derive(Debug, Clone)]
pub enum ParamType {
	Address,
	Bytes,
	Int,
	Uint,
	Bool,
	String,
	Array(Box<ParamType>),
	FixedBytes(usize),
	FixedArray(usize, Box<ParamType>),
}
