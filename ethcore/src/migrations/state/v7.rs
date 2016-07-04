use util::hash::{FixedHash, H256};
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
		let val_hash = value.sha3();
		assert!(key.len() == 32); // all keys in the state db are hashes.
		let key_h = H256::from_slice(&key[..]);
		if key_h != val_hash {
			// this is a key which has been xor'd with an address.
			// recover the address
			let address = key_h ^ val_hash;
			let address_hash = address.sha3();

			let new_key = address_hash ^ val_hash;
			key.copy_from_slice(&new_key[..]);
		}
		// nothing to do here
		Some((key, value))
	}
}

