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

