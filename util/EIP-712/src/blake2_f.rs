/// the precomputed values for BLAKE2b
/// there are 10 16-byte arrays - one for each round
/// the entries are calculated from the sigma constants.
const PRECOMPUTED: [[usize; 16]; 10] = [
	[0, 2, 4, 6, 1, 3, 5, 7, 8, 10, 12, 14, 9, 11, 13, 15],
	[14, 4, 9, 13, 10, 8, 15, 6, 1, 0, 11, 5, 12, 2, 7, 3],
	[11, 12, 5, 15, 8, 0, 2, 13, 10, 3, 7, 9, 14, 6, 1, 4],
	[7, 3, 13, 11, 9, 1, 12, 14, 2, 5, 4, 15, 6, 10, 0, 8],
	[9, 5, 2, 10, 0, 7, 4, 15, 14, 11, 6, 3, 1, 12, 8, 13],
	[2, 6, 0, 8, 12, 10, 11, 3, 4, 7, 15, 1, 13, 5, 14, 9],
	[12, 1, 14, 4, 5, 15, 13, 10, 0, 6, 9, 8, 7, 3, 2, 11],
	[13, 7, 12, 3, 11, 14, 1, 9, 5, 15, 8, 2, 0, 4, 6, 10],
	[6, 14, 11, 0, 15, 9, 3, 8, 12, 13, 1, 10, 2, 7, 4, 5],
	[10, 8, 7, 1, 2, 4, 6, 5, 15, 9, 3, 13, 11, 14, 12, 0],
];


/// IV is an initialization vector for BLAKE2b
const IV: [u64; 8] = [
	0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
	0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

/// F is a compression function for BLAKE2b. It takes as an argument the state
/// vector `h`, message block vector `m`, offset counter `t`, final
/// block indicator flag `f`, and number of rounds `rounds`. The state vector
/// provided as the first parameter is modified by the function.
fn blake2_f(h: &mut [u64; 8], m: [u64; 16], c: [u64; 2], f: bool, rounds: usize) {
	let (c0, c1) = (c[0], c[1]);

	let (
		mut v0,
		mut v1,
		mut v2,
		mut v3,
		mut v4,
		mut v5,
		mut v6,
		mut v7
	) = (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
	let (
		mut v8,
		mut v9,
		mut v10,
		mut v11,
		mut v12,
		mut v13,
		mut v14,
		mut v15
	) = (IV[0], IV[1], IV[2], IV[3], IV[4], IV[5], IV[6], IV[7]);
	v12 ^= c0;
	v13 ^= c1;

	if f {
		v14 ^= 0xffffffffffffffff;
	}

	for i in 0..rounds {
		let s = &(PRECOMPUTED[i % 10]);

		v0 = v0.overflowing_add(m[s[0]]).0;
		v0 = v0.overflowing_add(v4).0;
		v12 ^= v0;
		v12 = v12.rotate_right(32);
		v8 = v8.overflowing_add(v12).0;
		v4 ^= v8;
		v4 = v4.rotate_right(24);
		v1 = v1.overflowing_add(m[s[1]]).0;
		v1 = v1.overflowing_add(v5).0;
		v13 ^= v1;
		v13 = v13.rotate_right(32);
		v9 = v9.overflowing_add(v13).0;
		v5 ^= v9;
		v5 = v5.rotate_right(24);
		v2 = v2.overflowing_add(m[s[2]]).0;
		v2 = v2.overflowing_add(v6).0;
		v14 ^= v2;
		v14 = v14.rotate_right(32);
		v10 = v10.overflowing_add(v14).0;
		v6 ^= v10;
		v6 = v6.rotate_right(24);
		v3 = v3.overflowing_add(m[s[3]]).0;
		v3 = v3.overflowing_add(v7).0;
		v15 ^= v3;
		v15 = v15.rotate_right(32);
		v11 = v11.overflowing_add(v15).0;
		v7 ^= v11;
		v7 = v7.rotate_right(24);

		v0 = v0.overflowing_add(m[s[4]]).0;
		v0 = v0.overflowing_add(v4).0;
		v12 ^= v0;
		v12 = v12.rotate_right(16);
		v8 = v8.overflowing_add(v12).0;
		v4 ^= v8;
		v4 = v4.rotate_right(63);
		v1 = v1.overflowing_add(m[s[5]]).0;
		v1 = v1.overflowing_add(v5).0;
		v13 ^= v1;
		v13 = v13.rotate_right(16);
		v9 = v9.overflowing_add(v13).0;
		v5 ^= v9;
		v5 = v5.rotate_right(63);
		v2 = v2.overflowing_add(m[s[6]]).0;
		v2 = v2.overflowing_add(v6).0;
		v14 ^= v2;
		v14 = v14.rotate_right(16);
		v10 = v10.overflowing_add(v14).0;
		v6 ^= v10;
		v6 = v6.rotate_right(63);
		v3 = v3.overflowing_add(m[s[7]]).0;
		v3 = v3.overflowing_add(v7).0;
		v15 ^= v3;
		v15 = v15.rotate_right(16);
		v11 = v11.overflowing_add(v15).0;
		v7 ^= v11;
		v7 = v7.rotate_right(63);

		v0 = v0.overflowing_add(m[s[8]]).0;
		v0 = v0.overflowing_add(v5).0;
		v15 ^= v0;
		v15 = v15.rotate_right(32);
		v10 = v10.overflowing_add(v15).0;
		v5 ^= v10;
		v5 = v5.rotate_right(24);
		v1 = v1.overflowing_add(m[s[9]]).0;
		v1 = v1.overflowing_add(v6).0;
		v12 ^= v1;
		v12 = v12.rotate_right(32);
		v11 = v11.overflowing_add(v12).0;
		v6 ^= v11;
		v6 = v6.rotate_right(24);
		v2 = v2.overflowing_add(m[s[10]]).0;
		v2 = v2.overflowing_add(v7).0;
		v13 ^= v2;
		v13 = v13.rotate_right(32);
		v8 = v8.overflowing_add(v13).0;
		v7 ^= v8;
		v7 = v7.rotate_right(24);
		v3 = v3.overflowing_add(m[s[11]]).0;
		v3 = v3.overflowing_add(v4).0;
		v14 ^= v3;
		v14 = v14.rotate_right(32);
		v9 = v9.overflowing_add(v14).0;
		v4 ^= v9;
		v4 = v4.rotate_right(24);

		v0 = v0.overflowing_add(m[s[12]]).0;
		v0 = v0.overflowing_add(v5).0;
		v15 ^= v0;
		v15 = v15.rotate_right(16);
		v10 = v10.overflowing_add(v15).0;
		v5 ^= v10;
		v5 = v5.rotate_right(63);
		v1 = v1.overflowing_add(m[s[13]]).0;
		v1 = v1.overflowing_add(v6).0;
		v12 ^= v1;
		v12 = v12.rotate_right(16);
		v11 = v11.overflowing_add(v12).0;
		v6 ^= v11;
		v6 = v6.rotate_right(63);
		v2 = v2.overflowing_add(m[s[14]]).0;
		v2 = v2.overflowing_add(v7).0;
		v13 ^= v2;
		v13 = v13.rotate_right(16);
		v8 = v8.overflowing_add(v13).0;
		v7 ^= v8;
		v7 = v7.rotate_right(63);
		v3 = v3.overflowing_add(m[s[15]]).0;
		v3 = v3.overflowing_add(v4).0;
		v14 ^= v3;
		v14 = v14.rotate_right(16);
		v9 = v9.overflowing_add(v14).0;
		v4 ^= v9;
		v4 = v4.rotate_right(63);
	}

	h[0] ^= v0 ^ v8;
	h[1] ^= v1 ^ v9;
	h[2] ^= v2 ^ v10;
	h[3] ^= v3 ^ v11;
	h[4] ^= v4 ^ v12;
	h[5] ^= v5 ^ v13;
	h[6] ^= v6 ^ v14;
	h[7] ^= v7 ^ v15;
}

#[cfg(test)]
mod tests {
	use crate::blake2_f::blake2_f;

	#[test]
	fn test_blake2_f() {
		let mut h_in = [
			0x6a09e667f2bdc948_u64, 0xbb67ae8584caa73b_u64,
			0x3c6ef372fe94f82b_u64, 0xa54ff53a5f1d36f1_u64,
			0x510e527fade682d1_u64, 0x9b05688c2b3e6c1f_u64,
			0x1f83d9abfb41bd6b_u64, 0x5be0cd19137e2179_u64,
		];

		let m = [
			0x0000000000636261_u64, 0x0000000000000000_u64, 0x0000000000000000_u64,
			0x0000000000000000_u64, 0x0000000000000000_u64, 0x0000000000000000_u64,
			0x0000000000000000_u64, 0x0000000000000000_u64, 0x0000000000000000_u64,
			0x0000000000000000_u64, 0x0000000000000000_u64, 0x0000000000000000_u64,
			0x0000000000000000_u64, 0x0000000000000000_u64, 0x0000000000000000_u64,
			0x0000000000000000_u64,
		];
		let c = [3, 0];
		let f = true;
		let rounds = 12;
		let h_out: [u64; 8] = [
			0x0D4D1C983FA580BA_u64, 0xE9F6129FB697276A_u64, 0xB7C45A68142F214C_u64,
			0xD1A2FFDB6FBB124B_u64, 0x2D79AB2A39C5877D_u64, 0x95CC3345DED552C2_u64,
			0x5A92F1DBA88AD318_u64, 0x239900D4ED8623B9_u64,
		];

		blake2_f(&mut h_in, m, c, f, rounds);

		assert_eq!(h_in, h_out);
	}
}

