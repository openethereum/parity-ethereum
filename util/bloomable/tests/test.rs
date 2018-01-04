extern crate keccak_hash as hash;
extern crate ethcore_bigint;
extern crate bloomable;

use ethcore_bigint::hash::{H160, H256, H2048};
use bloomable::Bloomable;
use hash::keccak;

#[test]
fn shift_bloomed() {
	let bloom: H2048 = "00000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002020000000000000000000000000000000000000000000008000000001000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".into();
	let address: H160 = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
	let topic: H256 = "02c69be41d0b7e40352fc85be1cd65eb03d40ef8427a0ca4596b1ead9a00e9fc".into();

	let mut my_bloom = H2048::default();
	assert!(!my_bloom.contains_bloomed(&keccak(&address)));
	assert!(!my_bloom.contains_bloomed(&keccak(&topic)));

	my_bloom.shift_bloomed(&keccak(&address));
	assert!(my_bloom.contains_bloomed(&keccak(&address)));
	assert!(!my_bloom.contains_bloomed(&keccak(&topic)));

	my_bloom.shift_bloomed(&keccak(&topic));
	assert_eq!(my_bloom, bloom);
	assert!(my_bloom.contains_bloomed(&keccak(&address)));
	assert!(my_bloom.contains_bloomed(&keccak(&topic)));
}
