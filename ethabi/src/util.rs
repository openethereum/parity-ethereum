//! Utils used by different modules.

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
