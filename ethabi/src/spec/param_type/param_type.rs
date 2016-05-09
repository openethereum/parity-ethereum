//! Function and event param types.

/// Function and event param types.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
	/// Address.
	Address,
	/// Bytes.
	Bytes,
	/// Signed integer.
	Int(usize),
	/// Unisgned integer.
	Uint(usize),
	/// Boolean.
	Bool,
	/// String.
	String,
	/// Array of unknown size.
	Array(Box<ParamType>),
	/// Vector of bytes with fixed size.
	FixedBytes(usize),
	/// Array with fixed size.
	FixedArray(Box<ParamType>, usize),
}


