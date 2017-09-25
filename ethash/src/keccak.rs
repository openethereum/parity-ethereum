extern crate hash;

pub type H256 = [u8; 32];

pub mod keccak_512 {
	use super::hash;

	pub use self::hash::keccak_256 as unchecked;

	pub fn write(input: &[u8], output: &mut [u8]) {
		unsafe { hash::keccak_512(output.as_mut_ptr(), output.len(), input.as_ptr(), input.len()) };
	}

	pub fn inplace(input: &mut [u8]) {
		// This is safe since `sha3_*` uses an internal buffer and copies the result to the output. This
		// means that we can reuse the input buffer for both input and output.
		unsafe { hash::keccak_512(input.as_mut_ptr(), input.len(), input.as_ptr(), input.len()) };
	}
}

pub mod keccak_256 {
	use super::hash;

	pub use self::hash::keccak_256 as unchecked;

	#[allow(dead_code)]
	pub fn write(input: &[u8], output: &mut [u8]) {
		unsafe { hash::keccak_256(output.as_mut_ptr(), output.len(), input.as_ptr(), input.len()) };
	}

	pub fn inplace(input: &mut [u8]) {
		// This is safe since `sha3_*` uses an internal buffer and copies the result to the output. This
		// means that we can reuse the input buffer for both input and output.
		unsafe { hash::keccak_256(input.as_mut_ptr(), input.len(), input.as_ptr(), input.len()) };
	}
}
