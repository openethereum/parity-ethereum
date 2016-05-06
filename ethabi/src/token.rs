/// Ethereum ABI params.
#[derive(Debug, PartialEq)]
pub enum Token {
	/// Address.
	/// 
	/// solidity name: address
	/// Encoded to left padded [0u8; 32].
	Address([u8;20]),
	/// Vector of bytes with known size.
	/// 
	/// solidity name eg.: bytes8, bytes32, bytes64, bytes1024
	/// Encoded to right padded [0u8; ((N + 31) / 32) * 32].
	FixedBytes(Vec<u8>),
	/// Vector of bytes of unknown size.
	/// 
	/// solidity name: bytes
	/// Encoded in two parts.
	/// Init part: offset of 'closing part`.
	/// Closing part: encoded length followed by encoded right padded bytes.
	Bytes(Vec<u8>),
	/// Signed integer.
	/// 
	/// solidity name: int
	Int([u8;32]),
	/// Unisnged integer.
	/// 
	/// solidity name: uint
	Uint([u8;32]),
	/// Boolean value.
	/// 
	/// solidity name: bool
	/// Encoded as left padded [0u8; 32], where last bit represents boolean value.
	Bool(bool),
	/// String.
	/// 
	/// solidity name: string
	/// Encoded in the same way as bytes. Must be utf8 compliant.
	String(String),
	/// Array with known size.
	/// 
	/// solidity name eg.: int[3], bool[3], address[][8]
	/// Encoding of array is equal to encoding of consecutive elements of array.
	FixedArray(Vec<Token>),
	/// Array of params with unknown size.
	/// 
	/// solidity name eg. int[], bool[], address[5][]
	Array(Vec<Token>),
}
