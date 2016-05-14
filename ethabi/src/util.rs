use std::ptr;
use error::Error;

/// Convers vector of bytes with len equal n * 32, to a vector of slices.
pub fn slice_data(data: Vec<u8>) -> Result<Vec<[u8; 32]>, Error> {
	if data.len() % 32 != 0 {
		return Err(Error::InvalidData);
	}

	let times = data.len() / 32;
	let mut result = vec![];
	for i in 0..times {
		let mut slice = [0u8; 32];
		unsafe {
			ptr::copy(data.as_ptr().offset(32 * i as isize), slice.as_mut_ptr(), 32);
		}
		result.push(slice);
	}
	Ok(result)
}

/// Reads string as 32 bytes len slice. Unsafe.
#[cfg(test)]
pub fn read32(s: &str) -> [u8; 32] {
	use rustc_serialize::hex::FromHex;

	let mut result = [0u8; 32];
	let bytes = s.from_hex().unwrap();
	
	if bytes.len() != 32 {
		panic!();
	}
	
	unsafe {
		ptr::copy(bytes.as_ptr(), result.as_mut_ptr(), 32);
	}

	result
}

/// Reads string as 20 bytes len slice. Unsafe.
#[cfg(test)]
pub fn read20(s: &str) -> [u8; 20] {
	use rustc_serialize::hex::FromHex;

	let mut result = [0u8; 20];
	let bytes = s.from_hex().unwrap();
	
	if bytes.len() != 20 {
		panic!();
	}
	
	unsafe {
		ptr::copy(bytes.as_ptr(), result.as_mut_ptr(), 20);
	}

	result
}
