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

//! AVX2 implementation of the blake2b compression function.
use crate::IV;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use arrayref::{array_refs, mut_array_refs};

// Adapted from https://github.com/rust-lang-nursery/stdsimd/pull/479.
macro_rules! _MM_SHUFFLE {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

/// The Blake2b compression function F. See https://tools.ietf.org/html/rfc7693#section-3.2
/// Takes as an argument the state vector `state`, message block vector `message`, offset counter, final
/// block indicator flag `f`, and number of rounds `rounds`. The state vector provided as the first
/// parameter is modified by the function.
///
/// `g1` only operates on `x` from the original g function.
///  ```
/// fn portable_g1(v: &mut [u64], a: usize, b: usize, c: usize, d: usize, x: u64) {
///		v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
///		v[d] = (v[d] ^ v[a]).rotate_right(32);
///		v[c] = v[c].wrapping_add(v[d]);
///		v[b] = (v[b] ^ v[c]).rotate_right(24);
/// }
/// ```
///
/// `g2` only operates on `y` from the originial g function.
/// ```
/// fn portable_g2(v: &mut [u64], a: usize, b: usize, c: usize, d: usize, y: u64) {
///		v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
///		v[d] = (v[d] ^ v[a]).rotate_right(16);
///		v[c] = v[c].wrapping_add(v[d]);
///		v[b] = (v[b] ^ v[c]).rotate_right(63);
/// }
/// ```
///
/// Message mixing is done based on sigma values, for a given round.
///
/// # Example
///
/// `SIGMA` for round 1 i.e `SIGMA[0]` = `[ 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15]`;
/// ```
///  let s = &SIGMA[0 % 10];
/// //        a, b, c, d,    x
/// g(&mut v, 0, 4, 8 , 12, m[s[0]]);
///	g(&mut v, 1, 5, 9 , 13, m[s[2]]);
///	g(&mut v, 2, 6, 10, 14, m[s[4]]);
///	g(&mut v, 3, 7, 11, 15, m[s[6]]);
///
/// let a = v[..4];
/// let b = v[4..8];
/// let c = v[8..12];
/// let d = v[12..16];
/// let mut b0 = [m[0], m[2], m[4], m[6]];
///
///  g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
/// // ... then contruct b0 for `g2` etc.
/// ```
///
#[target_feature(enable = "avx2")]
pub unsafe fn compress(state: &mut [u64; 8], message: [u64; 16], count: [u64; 2], f: bool, rounds: usize) {
	// get a mutable reference to state[0..4], state[4..]
	let (state_low, state_high) = mut_array_refs!(state, 4, 4);
	// get a reference to IV[0..4], IV[4..]
	let (iv_low, iv_high) = array_refs!(&IV, 4, 4);

	// loads them into an __m256i
	let mut a = loadu(state_low);
	let mut b = loadu(state_high);
	let mut c = loadu(iv_low);

	// !a = xor(a, xor(a, !a))
	let inverse = if f {
		iv_high[3] ^ !iv_high[3]
	} else {
		0
	};

	let flags = set4(
		count[0],
		count[1],
		inverse,
		0,
	);

	let mut d = xor(loadu(iv_high), flags);

	// get a reference to message[(0..2)+,]
	let msg_chunks = array_refs!(&message, 2, 2, 2, 2, 2, 2, 2, 2);
	// load each message [u64; 2] into an __m128i, broadcast it into both lanes of an __m256i.

	// m0 = __m256i([message[0], message[1], message[0], message[1]])
	let m0 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.0));
	// m1 = __m256i([message[2], message[3], message[2], message[3]])
	let m1 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.1));
	// m2 = __m256i([message[4], message[5], message[4], message[5]])
	let m2 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.2));
	// m3 = __m256i([message[6], message[7], message[6], message[7]])
	let m3 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.3));
	// m4 = __m256i([message[8], message[9], message[8], message[9]])
	let m4 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.4));
	// m5 = __m256i([message[10], message[11], message[10], message[11]])
	let m5 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.5));
	// m6 = __m256i([message[12], message[13], message[12], message[13]])
	let m6 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.6));
	// m7 = __m256i([message[14], message[15], message[14], message[15]])
	let m7 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.7));

	let iv0 = a;
	let iv1 = b;

	let mut t0;
	let mut t1;
	let mut b0;

	for i in 0..rounds {
		match i % 10 {
			0 => {
				t0 = _mm256_unpacklo_epi64(m0, m1); // ([0, 1, 0, 1], [2, 3, 2, 3]) = [0, 2, 0, 2]
				t1 = _mm256_unpacklo_epi64(m2, m3); // ([4, 5, 4, 5], [6, 7, 6, 7]) = [4, 6, 4, 6]
				b0 = _mm256_blend_epi32(t0, t1, 0xF0); // ([0, 2, 0, 2], [4, 6, 4, 6]) = [0, 2, 4, 6]
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpackhi_epi64(m0, m1); // ([0, 1, 0, 1], [2, 3, 2, 3]) = [1, 3, 1, 3]
				t1 = _mm256_unpackhi_epi64(m2, m3); // ([4, 5, 4, 5], [6, 7, 6, 7]) = [5, 7, 5, 7]
				b0 = _mm256_blend_epi32(t0, t1, 0xF0); // ([1, 3, 1, 3], [5, 7, 5, 7]) = [1, 3, 5, 7]
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_unpacklo_epi64(m7, m4); // ([14, 15, 14, 15], [8, 9, 8, 9]) = [14, 8, 14, 8]
				t1 = _mm256_unpacklo_epi64(m5, m6); // ([10, 11, 10, 11], [12, 13, 12, 13]) = [10, 12, 10, 12]
				b0 = _mm256_blend_epi32(t0, t1, 0xF0); // ([14, 8, 14, 8], [10, 12, 10, 12]) = [14, 8, 10, 12]
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpackhi_epi64(m7, m4); // ([14, 15, 14, 15], [8, 9, 8, 9]) = [15, 9, 15, 9]
				t1 = _mm256_unpackhi_epi64(m5, m6); // ([10, 11, 10, 11], [12, 13, 12, 13]) = [11, 13, 11, 13]
				b0 = _mm256_blend_epi32(t0, t1, 0xF0); // ([15, 9, 15, 9], [11, 13, 11, 13]) = [15, 9, 11, 13]
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			1 => {
				t0 = _mm256_unpacklo_epi64(m7, m2);
				t1 = _mm256_unpackhi_epi64(m4, m6);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpacklo_epi64(m5, m4);
				t1 = _mm256_alignr_epi8(m3, m7, 8);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_unpackhi_epi64(m2, m0);
				t1 = _mm256_blend_epi32(m5, m0, 0x33);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_alignr_epi8(m6, m1, 8);
				t1 = _mm256_blend_epi32(m3, m1, 0x33);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			2 => {
				// round 3
				t0 = _mm256_alignr_epi8(m6, m5, 8);
				t1 = _mm256_unpackhi_epi64(m2, m7);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpacklo_epi64(m4, m0);
				t1 = _mm256_blend_epi32(m6, m1, 0x33);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_alignr_epi8(m5, m4, 8);
				t1 = _mm256_unpackhi_epi64(m1, m3);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpacklo_epi64(m2, m7);
				t1 = _mm256_blend_epi32(m0, m3, 0x33);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			3 => {
				// round 4
				t0 = _mm256_unpackhi_epi64(m3, m1);
				t1 = _mm256_unpackhi_epi64(m6, m5);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpackhi_epi64(m4, m0);
				t1 = _mm256_unpacklo_epi64(m6, m7);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_alignr_epi8(m1, m7, 8);
				t1 = _mm256_shuffle_epi32(m2, _MM_SHUFFLE!(1, 0, 3, 2));
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpacklo_epi64(m4, m3);
				t1 = _mm256_unpacklo_epi64(m5, m0);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			4 => {
				// round 5
				t0 = _mm256_unpackhi_epi64(m4, m2);
				t1 = _mm256_unpacklo_epi64(m1, m5);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_blend_epi32(m3, m0, 0x33);
				t1 = _mm256_blend_epi32(m7, m2, 0x33);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_alignr_epi8(m7, m1, 8);
				t1 = _mm256_alignr_epi8(m3, m5, 8);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpackhi_epi64(m6, m0);
				t1 = _mm256_unpacklo_epi64(m6, m4);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			5 => {
				// round 6
				t0 = _mm256_unpacklo_epi64(m1, m3);
				t1 = _mm256_unpacklo_epi64(m0, m4);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpacklo_epi64(m6, m5);
				t1 = _mm256_unpackhi_epi64(m5, m1);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_alignr_epi8(m2, m0, 8);
				t1 = _mm256_unpackhi_epi64(m3, m7);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpackhi_epi64(m4, m6);
				t1 = _mm256_alignr_epi8(m7, m2, 8);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			6 => {
				// round 7
				t0 = _mm256_blend_epi32(m0, m6, 0x33);
				t1 = _mm256_unpacklo_epi64(m7, m2);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpackhi_epi64(m2, m7);
				t1 = _mm256_alignr_epi8(m5, m6, 8);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_unpacklo_epi64(m4, m0);
				t1 = _mm256_blend_epi32(m4, m3, 0x33);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpackhi_epi64(m5, m3);
				t1 = _mm256_shuffle_epi32(m1, _MM_SHUFFLE!(1, 0, 3, 2));
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			7 => {
				// round 8
				t0 = _mm256_unpackhi_epi64(m6, m3);
				t1 = _mm256_blend_epi32(m1, m6, 0x33);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_alignr_epi8(m7, m5, 8);
				t1 = _mm256_unpackhi_epi64(m0, m4);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_blend_epi32(m2, m1, 0x33);
				t1 = _mm256_alignr_epi8(m4, m7, 8);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpacklo_epi64(m5, m0);
				t1 = _mm256_unpacklo_epi64(m2, m3);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			8 => {
				// round 9
				t0 = _mm256_unpacklo_epi64(m3, m7);
				t1 = _mm256_alignr_epi8(m0, m5, 8);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpackhi_epi64(m7, m4);
				t1 = _mm256_alignr_epi8(m4, m1, 8);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_unpacklo_epi64(m5, m6);
				t1 = _mm256_unpackhi_epi64(m6, m0);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_alignr_epi8(m1, m2, 8);
				t1 = _mm256_alignr_epi8(m2, m3, 8);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
			_ => {
				// round 10
				t0 = _mm256_unpacklo_epi64(m5, m4);
				t1 = _mm256_unpackhi_epi64(m3, m0);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_unpacklo_epi64(m1, m2);
				t1 = _mm256_blend_epi32(m2, m3, 0x33);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				diagonalize(&mut a, &mut b, &mut c, &mut d);
				t0 = _mm256_unpackhi_epi64(m6, m7);
				t1 = _mm256_unpackhi_epi64(m4, m1);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
				t0 = _mm256_blend_epi32(m5, m0, 0x33);
				t1 = _mm256_unpacklo_epi64(m7, m6);
				b0 = _mm256_blend_epi32(t0, t1, 0xF0);
				g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
				undiagonalize(&mut a, &mut b, &mut c, &mut d);
			}
		}
	}

	a = xor(a, c);
	b = xor(b, d);
	a = xor(a, iv0);
	b = xor(b, iv1);

	storeu(a, state_low);
	storeu(b, state_high);
}


#[inline(always)]
unsafe fn loadu(src: *const [u64; 4]) -> __m256i {
	// This is an unaligned load, so the pointer cast is allowed.
	_mm256_loadu_si256(src as *const __m256i)
}

#[inline(always)]
unsafe fn storeu(src: __m256i, dest: *mut [u64; 4]) {
	// This is an unaligned store, so the pointer cast is allowed.
	_mm256_storeu_si256(dest as *mut __m256i, src)
}

#[inline(always)]
unsafe fn loadu_128(mem_addr: &[u64; 2]) -> __m128i {
	_mm_loadu_si128(mem_addr.as_ptr() as *const __m128i)
}

#[inline(always)]
unsafe fn add(a: __m256i, b: __m256i) -> __m256i {
	_mm256_add_epi64(a, b)
}

#[inline(always)]
unsafe fn xor(a: __m256i, b: __m256i) -> __m256i {
	_mm256_xor_si256(a, b)
}

#[inline(always)]
unsafe fn set4(a: u64, b: u64, c: u64, d: u64) -> __m256i {
	_mm256_setr_epi64x(a as i64, b as i64, c as i64, d as i64)
}

#[inline(always)]
unsafe fn rotate_right_32(x: __m256i) -> __m256i {
	_mm256_shuffle_epi32(x, _MM_SHUFFLE!(2, 3, 0, 1))
}

#[inline(always)]
unsafe fn rotate_right_24(x: __m256i) -> __m256i {
	let rotate24 = _mm256_setr_epi8(
		3, 4, 5, 6, 7, 0, 1, 2, 11, 12, 13, 14, 15, 8, 9, 10, 3, 4, 5, 6, 7, 0, 1, 2, 11, 12, 13,
		14, 15, 8, 9, 10,
	);
	_mm256_shuffle_epi8(x, rotate24)
}

#[inline(always)]
unsafe fn rotate_right_16(x: __m256i) -> __m256i {
	let rotate16 = _mm256_setr_epi8(
		2, 3, 4, 5, 6, 7, 0, 1, 10, 11, 12, 13, 14, 15, 8, 9, 2, 3, 4, 5, 6, 7, 0, 1, 10, 11, 12,
		13, 14, 15, 8, 9,
	);
	_mm256_shuffle_epi8(x, rotate16)
}

#[inline(always)]
unsafe fn rotate_right_63(x: __m256i) -> __m256i {
	_mm256_or_si256(_mm256_srli_epi64(x, 63), add(x, x))
}

#[inline(always)]
unsafe fn g1(a: &mut __m256i, b: &mut __m256i, c: &mut __m256i, d: &mut __m256i, m: &mut __m256i) {
	*a = add(*a, *m);
	*a = add(*a, *b);
	*d = xor(*d, *a);
	*d = rotate_right_32(*d);
	*c = add(*c, *d);
	*b = xor(*b, *c);
	*b = rotate_right_24(*b);
}

#[inline(always)]
unsafe fn g2(a: &mut __m256i, b: &mut __m256i, c: &mut __m256i, d: &mut __m256i, m: &mut __m256i) {
	*a = add(*a, *m);
	*a = add(*a, *b);
	*d = xor(*d, *a);
	*d = rotate_right_16(*d);
	*c = add(*c, *d);
	*b = xor(*b, *c);
	*b = rotate_right_63(*b);
}

// Note the optimization here of leaving b as the unrotated row, rather than a.
// All the message loads below are adjusted to compensate for this. See
// discussion at https://github.com/sneves/blake2-avx2/pull/4
#[inline(always)]
unsafe fn diagonalize(a: &mut __m256i, _b: &mut __m256i, c: &mut __m256i, d: &mut __m256i) {
	*a = _mm256_permute4x64_epi64(*a, _MM_SHUFFLE!(2, 1, 0, 3));
	*d = _mm256_permute4x64_epi64(*d, _MM_SHUFFLE!(1, 0, 3, 2));
	*c = _mm256_permute4x64_epi64(*c, _MM_SHUFFLE!(0, 3, 2, 1));
}

// Note the optimization here of leaving b as the unrotated row, rather than a.
// All the message loads below are adjusted to compensate for this. See
// discussion at https://github.com/sneves/blake2-avx2/pull/4
#[inline(always)]
unsafe fn undiagonalize(a: &mut __m256i, _b: &mut __m256i, c: &mut __m256i, d: &mut __m256i) {
	*a = _mm256_permute4x64_epi64(*a, _MM_SHUFFLE!(0, 3, 2, 1));
	*d = _mm256_permute4x64_epi64(*d, _MM_SHUFFLE!(1, 0, 3, 2));
	*c = _mm256_permute4x64_epi64(*c, _MM_SHUFFLE!(2, 1, 0, 3));
}


#[cfg(test)]
mod tests {
	#[test]
	fn test_mm_shuffle() {
		assert_eq!(_MM_SHUFFLE!(0, 1, 1, 3), 0b00_01_01_11);
		assert_eq!(_MM_SHUFFLE!(3, 1, 1, 0), 0b11_01_01_00);
		assert_eq!(_MM_SHUFFLE!(1, 2, 2, 1), 0b01_10_10_01);
	}
}
