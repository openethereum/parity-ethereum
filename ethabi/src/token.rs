pub enum Token {
	Address([u8;20]),
	FixedBytes(Vec<u8>),
	Bytes(Vec<u8>),
	Int([u8;32]),
	Uint([u8;32]),
	Bool(bool),
	String(String),
	FixedArray(Vec<Token>),
	Array(Vec<Token>)
}
