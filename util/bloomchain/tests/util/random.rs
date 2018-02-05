extern crate rand;

use self::rand::random;
use bloomchain::Bloom;

pub fn generate_random_bloom() -> Bloom {
	let mut res = [0u8; 256];
	let p0 = random::<u8>();
	let b0 = random::<u8>() % 8;
	let p1 = random::<u8>();
	let b1 = random::<u8>() % 8;
	let p2 = random::<u8>();
	let b2 = random::<u8>() % 8;

	res[p0 as usize] |= 1 << b0;
	res[p1 as usize] |= 1 << b1;
	res[p2 as usize] |= 1 << b2;
	
	From::from(res)
}

pub fn generate_n_random_blooms(n: usize) -> Vec<Bloom> {
	(0..n).map(|_| generate_random_bloom()).collect()
}
