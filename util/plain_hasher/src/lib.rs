#![no_std]
#[macro_use]
extern crate crunchy;

use core::{hash, mem};

/// Hasher that just takes 8 bytes of the provided value.
/// May only be used for keys which are 32 bytes.
#[derive(Default)]
pub struct PlainHasher {
	prefix: u64,
}

impl hash::Hasher for PlainHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.prefix
	}

	#[inline]
	#[allow(unused_assignments)]
	fn write(&mut self, bytes: &[u8]) {
		debug_assert!(bytes.len() == 32);

		unsafe {
			let mut bytes_ptr = bytes.as_ptr();
			let prefix_u8: &mut [u8; 8] = mem::transmute(&mut self.prefix);
			let mut prefix_ptr = prefix_u8.as_mut_ptr();

			unroll! {
				for _i in 0..8 {
					*prefix_ptr ^= (*bytes_ptr ^ *bytes_ptr.offset(8)) ^ (*bytes_ptr.offset(16) ^ *bytes_ptr.offset(24));

					bytes_ptr = bytes_ptr.offset(1);
					prefix_ptr = prefix_ptr.offset(1);
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use core::hash::Hasher;
	use super::PlainHasher;

	#[test]
	fn it_works() {
		let mut bytes = [32u8; 32];
		bytes[0] = 15;
		let mut hasher = PlainHasher::default();
		hasher.write(&bytes);
		assert_eq!(hasher.prefix, 47);
	}
}
