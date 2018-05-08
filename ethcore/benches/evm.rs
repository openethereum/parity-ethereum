// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

#![feature(test)]

extern crate test;
extern crate ethcore_util as util;
extern crate rand;
extern crate bn;
extern crate ethcore_crypto;
extern crate ethkey;
extern crate rustc_hex;
extern crate ethcore_bigint;

use self::test::{Bencher};
use rand::{StdRng};


#[bench]
fn bn_128_pairing(b: &mut Bencher) {
	use bn::{pairing, G1, G2, Fr, Group};

	let rng = &mut ::rand::thread_rng();

	let sk0 = Fr::random(rng);
	let sk1 = Fr::random(rng);

	let pk0 = G1::one() * sk0;
	let pk1 = G2::one() * sk1;

	b.iter(|| {
		let _ = pairing(pk0, pk1);
	});
}

#[bench]
fn bn_128_mul(b: &mut Bencher) {
	use bn::{AffineG1, G1, Fr, Group};

	let mut rng = StdRng::new().unwrap();
	let p: G1 = G1::random(&mut rng);
	let fr = Fr::random(&mut rng);

	b.iter(|| {
		let _ = AffineG1::from_jacobian(p * fr);
	});
}

#[bench]
fn sha256(b: &mut Bencher) {
	use ethcore_crypto::digest::sha256;

	let mut input: [u8; 256] = [0; 256];
	let mut out = [0; 32];

	b.iter(|| {
		sha256(&input);
	});
}

#[bench]
fn ecrecover(b: &mut Bencher) {
	use rustc_hex::FromHex;
	use ethkey::{Signature, recover as ec_recover};
	use ethcore_bigint::hash::H256;
	let input = FromHex::from_hex("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();
	let hash = H256::from_slice(&input[0..32]);
	let v = H256::from_slice(&input[32..64]);
	let r = H256::from_slice(&input[64..96]);
	let s = H256::from_slice(&input[96..128]);

	let bit = match v[31] {
		27 | 28 if &v.0[..31] == &[0; 31] => v[31] - 27,
		_ => { return; },
	};

	let s = Signature::from_rsv(&r, &s, bit);
	b.iter(|| {
		let _ = ec_recover(&s, &hash);
	});
}

