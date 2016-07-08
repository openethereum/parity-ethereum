use util::migration::SimpleMigration;
use util::rlp::*;

/// This migration compresses RLP values using a simple replacement scheme.
pub struct ToV7;

impl SimpleMigration for ToV7 {
	fn version(&self) -> u32 {
		7
	}

	fn simple_migrate(&self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		let rlp = UntrustedRlp::new(&value);
		Some((key, rlp.compress().to_vec()))
	}
}
