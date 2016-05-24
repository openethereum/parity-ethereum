use std::ptr;
use util::migration::Migration;

/// This migration reduces the sizes of keys and moves `ExtrasIndex` byte from back to the front.
pub struct Migrate6;

impl Migrate6 {
	fn migrate_old_key(&self, old_key: Vec<u8>, index: u8, len: usize) -> Vec<u8> {
		let mut result = vec![];
		result.reserve(len);
		unsafe {
			result.set_len(len);
			result[0] = index;
			ptr::copy(old_key.as_ptr().offset(33 - len as isize), result.as_mut_ptr().offset(1), len - 1);
		}
		result
	}
}

impl Migration for Migrate6 {
	fn version(&self) -> u32 {
		6
	}

	fn simple_migrate(&self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {

		//// at this version all extras keys are 33 bytes long.
		if key.len() == 33 {
			// block details key changes:
			// - index is moved to the front
			if key[32] == 0 {
				return Some((self.migrate_old_key(key, 0, 33), value));
			}

			// block hash key changes:
			// - key is shorter 33 -> 5 bytes
			// - index is moved to the front
			if key[32] == 1 {
				return Some((self.migrate_old_key(key, 1, 5), value));
			}

			// transaction addresses changes:
			// - index is moved to the front
			if key[32] == 2 {
				return Some((self.migrate_old_key(key, 2, 33), value));
			}

			// block log blooms are removed
			if key[32] == 3 {
				return None;
			}

			// blocks blooms key changes:
			// - key is shorter 33 -> 6 bytes
			// - index is moved to the front
			// - index is changed 4 -> 3
			if key[32] == 4 {
				// i have no idea why it was reversed
				let reverse = key.into_iter().rev().collect::<Vec<_>>();
				let mut result = [0u8; 6];
				// new extras index is 3
				result[0] = 3;
				// 8th bytes was the level. Not it's second.
				result[1] = reverse[8];
				unsafe {
					// position at given level. we assume, that bytes were left padded.
					ptr::copy(reverse.as_ptr().offset(4), result.as_mut_ptr().offset(2), 4);
				}
				return Some((result.to_vec(), value));
			}

			// blocks receipts key changes:
			// - index is moved to the front
			// - index is changed 5 -> 4
			if key[32] == 5 {
				return Some((self.migrate_old_key(key, 4, 33), value));
			}
		}

		Some((key, value))
	}
}

