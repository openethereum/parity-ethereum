// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Portable implementation of the blake2b compress function

use crate::{IV, SIGMA};

/// The G mixing function. See https://tools.ietf.org/html/rfc7693#section-3.1
#[inline(always)]
fn g(v: &mut [u64], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
	v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
	v[d] = (v[d] ^ v[a]).rotate_right(32);
	v[c] = v[c].wrapping_add(v[d]);
	v[b] = (v[b] ^ v[c]).rotate_right(24);

	v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
	v[d] = (v[d] ^ v[a]).rotate_right(16);
	v[c] = v[c].wrapping_add(v[d]);
	v[b] = (v[b] ^ v[c]).rotate_right(63);
}

/// The Blake2b compression function F. See https://tools.ietf.org/html/rfc7693#section-3.2
/// Takes as an argument the state vector `h`, message block vector `m`, offset counter `t`, final
/// block indicator flag `f`, and number of rounds `rounds`. The state vector provided as the first
/// parameter is modified by the function.
pub fn compress(h: &mut [u64; 8], m: [u64; 16], t: [u64; 2], f: bool, rounds: usize) {
	let mut v = [0u64; 16];
	v[..8].copy_from_slice(h);    // First half from state.
	v[8..].copy_from_slice(&IV);  // Second half from IV.

	v[12] ^= t[0];
	v[13] ^= t[1];

	if f {
		v[14] = !v[14]; // Invert all bits if the last-block-flag is set.
	}

	for i in 0..rounds {
		// Message word selection permutation for this round.
		let s = &SIGMA[i % 10];
		g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
		g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
		g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
		g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);

		g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
		g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
		g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
		g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
	}

	for i in 0..8 {
		h[i] ^= v[i] ^ v[i + 8];
	}
}
