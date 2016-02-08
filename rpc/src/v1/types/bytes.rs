use rustc_serialize::hex::ToHex;
use serde::{Serialize, Serializer};

/// Wrapper structure around vector of bytes.
#[derive(Debug)]
pub struct Bytes(Vec<u8>);

impl Bytes {
	/// Simple constructor.
	pub fn new(bytes: Vec<u8>) -> Bytes {
		Bytes(bytes)
	}
}

impl Default for Bytes {
	fn default() -> Self {
		// default serialized value is 0x00
		Bytes(vec![0])
	}
}

impl Serialize for Bytes {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> 
	where S: Serializer {
		let mut serialized = "0x".to_owned();
		serialized.push_str(self.0.to_hex().as_ref());
		serializer.visit_str(serialized.as_ref())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;
	use rustc_serialize::hex::FromHex;

	#[test]
	fn test_bytes_serialize() {
		let bytes = Bytes("0123456789abcdef".from_hex().unwrap());
		let serialized = serde_json::to_string(&bytes).unwrap();
		assert_eq!(serialized, r#""0x0123456789abcdef""#);
	}
}


