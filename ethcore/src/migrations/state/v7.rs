use util::hash::{Address, FixedHash, H256};
use util::migration::SimpleMigration;
use util::sha3::Hashable;

/// This migration migrates the state db to use an accountdb which ensures uniqueness
/// using an address' hash as opposed to the address itself.
pub struct ToV7;

impl SimpleMigration for ToV7 {
	fn version(&self) -> u32 {
		7
	}

	fn simple_migrate(&self, mut key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		if key.len() != 32 {
			// metadata key, ignore.
			return Some((key, value));
		}

		let val_hash = value.sha3();
		let key_h = H256::from_slice(&key[..]);
		if key_h != val_hash {
			// this is a key which has been xor'd with an address.
			// recover the address.
			let address = key_h ^ val_hash;

			// check that the address is actually a 20-byte value.
			// the leftmost 12 bytes should be zero.
			if &address[0..12] != &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] {
				// metadata key that was 32 bytes, ignore.
				return Some((key, value));
			}

			let address_hash = Address::from(address).sha3();

			// create the xor'd key in place.
			key.copy_from_slice(&*val_hash);
			assert_eq!(key, &*val_hash);

			let last_src: &[u8] = &*address_hash;
			for (k, a) in key[12..].iter_mut().zip(&last_src[12..]) {
				*k ^= *a;
			}
		}
		// nothing to do here
		Some((key, value))
	}
}

