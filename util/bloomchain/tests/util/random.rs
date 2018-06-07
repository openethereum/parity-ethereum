// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
