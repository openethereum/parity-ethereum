use rustc_serialize::hex::*;

#[derive(Debug)]
pub enum EthcoreError {
	FromHex(FromHexError),
	BadSize
}

impl From<FromHexError> for EthcoreError {
	fn from(err: FromHexError) -> EthcoreError {
		EthcoreError::FromHex(err)
	}
}
