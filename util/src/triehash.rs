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

//! Generetes trie root.
//!
//! This module should be used to generate trie root hash.

use std::collections::BTreeMap;
use std::cmp;
use hash::*;
use sha3::*;
use rlp;
use rlp::{RlpStream, Stream};
use vector::SharedPrefix;

/// Generates a trie root hash for a vector of values
///
/// ```rust
/// extern crate ethcore_util as util;
/// use std::str::FromStr;
/// use util::triehash::*;
/// use util::hash::*;
///
/// fn main() {
/// 	let v = vec![From::from("doe"), From::from("reindeer")];
/// 	let root = "e766d5d51b89dc39d981b41bda63248d7abce4f0225eefd023792a540bcffee3";
/// 	assert_eq!(ordered_trie_root(v), H256::from_str(root).unwrap());
/// }
/// ```
pub fn ordered_trie_root<I>(input: I) -> H256
	where I: IntoIterator<Item=Vec<u8>>
{
	let gen_input = input
		// first put elements into btree to sort them by nibbles
		// optimize it later
		.into_iter()
		.enumerate()
		.map(|(i, vec)| (rlp::encode(&i).to_vec(), vec))
		.collect::<BTreeMap<_, _>>()
		// then move them to a vector
		.into_iter()
		.map(|(k, v)| (as_nibbles(&k), v) )
		.collect();

	gen_trie_root(gen_input)
}

/// Generates a trie root hash for a vector of key-values
///
/// ```rust
/// extern crate ethcore_util as util;
/// use std::str::FromStr;
/// use util::triehash::*;
/// use util::hash::*;
///
/// fn main() {
/// 	let v = vec![
/// 		(From::from("doe"), From::from("reindeer")),
/// 		(From::from("dog"), From::from("puppy")),
/// 		(From::from("dogglesworth"), From::from("cat")),
/// 	];
///
/// 	let root = "8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3";
/// 	assert_eq!(trie_root(v), H256::from_str(root).unwrap());
/// }
/// ```
pub fn trie_root<I>(input: I) -> H256
	where I: IntoIterator<Item=(Vec<u8>, Vec<u8>)>
{
	let gen_input = input
		// first put elements into btree to sort them and to remove duplicates
		.into_iter()
		.collect::<BTreeMap<_, _>>()
		// then move them to a vector
		.into_iter()
		.map(|(k, v)| (as_nibbles(&k), v) )
		.collect();

	gen_trie_root(gen_input)
}

/// Generates a key-hashed (secure) trie root hash for a vector of key-values.
///
/// ```rust
/// extern crate ethcore_util as util;
/// use std::str::FromStr;
/// use util::triehash::*;
/// use util::hash::*;
///
/// fn main() {
/// 	let v = vec![
/// 		(From::from("doe"), From::from("reindeer")),
/// 		(From::from("dog"), From::from("puppy")),
/// 		(From::from("dogglesworth"), From::from("cat")),
/// 	];
///
/// 	let root = "d4cd937e4a4368d7931a9cf51686b7e10abb3dce38a39000fd7902a092b64585";
/// 	assert_eq!(sec_trie_root(v), H256::from_str(root).unwrap());
/// }
/// ```
pub fn sec_trie_root(input: Vec<(Vec<u8>, Vec<u8>)>) -> H256 {
	let gen_input = input
		// first put elements into btree to sort them and to remove duplicates
		.into_iter()
		.map(|(k, v)| (k.sha3().to_vec(), v))
		.collect::<BTreeMap<_, _>>()
		// then move them to a vector
		.into_iter()
		.map(|(k, v)| (as_nibbles(&k), v) )
		.collect();

	gen_trie_root(gen_input)
}

fn gen_trie_root(input: Vec<(Vec<u8>, Vec<u8>)>) -> H256 {
	let mut stream = RlpStream::new();
	hash256rlp(&input, 0, &mut stream);
	stream.out().sha3()
}

/// Hex-prefix Notation. First nibble has flags: oddness = 2^0 & termination = 2^1.
///
/// The "termination marker" and "leaf-node" specifier are completely equivalent.
///
/// Input values are in range `[0, 0xf]`.
///
/// ```markdown
///  [0,0,1,2,3,4,5]   0x10012345 // 7 > 4
///  [0,1,2,3,4,5]     0x00012345 // 6 > 4
///  [1,2,3,4,5]       0x112345   // 5 > 3
///  [0,0,1,2,3,4]     0x00001234 // 6 > 3
///  [0,1,2,3,4]       0x101234   // 5 > 3
///  [1,2,3,4]         0x001234   // 4 > 3
///  [0,0,1,2,3,4,5,T] 0x30012345 // 7 > 4
///  [0,0,1,2,3,4,T]   0x20001234 // 6 > 4
///  [0,1,2,3,4,5,T]   0x20012345 // 6 > 4
///  [1,2,3,4,5,T]     0x312345   // 5 > 3
///  [1,2,3,4,T]       0x201234   // 4 > 3
/// ```
fn hex_prefix_encode(nibbles: &[u8], leaf: bool) -> Vec<u8> {
	let inlen = nibbles.len();
	let oddness_factor = inlen % 2;
	// next even number divided by two
	let reslen = (inlen + 2) >> 1;
	let mut res = vec![];
	res.reserve(reslen);

	let first_byte = {
		let mut bits = ((inlen as u8 & 1) + (2 * leaf as u8)) << 4;
		if oddness_factor == 1 {
			bits += nibbles[0];
		}
		bits
	};

	res.push(first_byte);

	let mut offset = oddness_factor;
	while offset < inlen {
		let byte = (nibbles[offset] << 4) + nibbles[offset + 1];
		res.push(byte);
		offset += 2;
	}

	res
}

/// Converts slice of bytes to nibbles.
fn as_nibbles(bytes: &[u8]) -> Vec<u8> {
	let mut res = vec![];
	res.reserve(bytes.len() * 2);
	for i in 0..bytes.len() {
		res.push(bytes[i] >> 4);
		res.push((bytes[i] << 4) >> 4);
	}
	res
}

fn hash256rlp(input: &[(Vec<u8>, Vec<u8>)], pre_len: usize, stream: &mut RlpStream) {
	let inlen = input.len();

	// in case of empty slice, just append empty data
	if inlen == 0 {
		stream.append_empty_data();
		return;
	}

	// take slices
	let key: &[u8] = &input[0].0;
	let value: &[u8] = &input[0].1;

	// if the slice contains just one item, append the suffix of the key
	// and then append value
	if inlen == 1 {
		stream.begin_list(2);
		stream.append(&hex_prefix_encode(&key[pre_len..], true));
		stream.append(&value);
		return;
	}

	// get length of the longest shared prefix in slice keys
	let shared_prefix = input.iter()
		// skip first element
		.skip(1)
		// get minimum number of shared nibbles between first and each successive
		.fold(key.len(), | acc, &(ref k, _) | {
			cmp::min(key.shared_prefix_len(k), acc)
		});

	// if shared prefix is higher than current prefix append its
	// new part of the key to the stream
	// then recursively append suffixes of all items who had this key
	if shared_prefix > pre_len {
		stream.begin_list(2);
		stream.append(&hex_prefix_encode(&key[pre_len..shared_prefix], false));
		hash256aux(input, shared_prefix, stream);
		return;
	}

	// an item for every possible nibble/suffix
	// + 1 for data
	stream.begin_list(17);

	// if first key len is equal to prefix_len, move to next element
	let mut begin = match pre_len == key.len() {
		true => 1,
		false => 0
	};

	// iterate over all possible nibbles
	for i in 0..16 {
		// cout how many successive elements have same next nibble
		let len = match begin < input.len() {
			true => input[begin..].iter()
				.take_while(| pair | pair.0[pre_len] == i )
				.count(),
			false => 0
		};

		// if at least 1 successive element has the same nibble
		// append their suffixes
		match len {
			0 => { stream.append_empty_data(); },
			_ => hash256aux(&input[begin..(begin + len)], pre_len + 1, stream)
		}
		begin += len;
	}

	// if fist key len is equal prefix, append its value
	match pre_len == key.len() {
		true => { stream.append(&value); },
		false => { stream.append_empty_data(); }
	};
}

fn hash256aux(input: &[(Vec<u8>, Vec<u8>)], pre_len: usize, stream: &mut RlpStream) {
	let mut s = RlpStream::new();
	hash256rlp(input, pre_len, &mut s);
	let out = s.out();
	match out.len() {
		0...31 => stream.append_raw(&out, 1),
		_ => stream.append(&out.sha3())
	};
}


#[test]
fn test_nibbles() {
	let v = vec![0x31, 0x23, 0x45];
	let e = vec![3, 1, 2, 3, 4, 5];
	assert_eq!(as_nibbles(&v), e);

	// A => 65 => 0x41 => [4, 1]
	let v: Vec<u8> = From::from("A");
	let e = vec![4, 1];
	assert_eq!(as_nibbles(&v), e);
}

#[test]
fn test_hex_prefix_encode() {
	let v = vec![0, 0, 1, 2, 3, 4, 5];
	let e = vec![0x10, 0x01, 0x23, 0x45];
	let h = hex_prefix_encode(&v, false);
	assert_eq!(h, e);

	let v = vec![0, 1, 2, 3, 4, 5];
	let e = vec![0x00, 0x01, 0x23, 0x45];
	let h = hex_prefix_encode(&v, false);
	assert_eq!(h, e);

	let v = vec![0, 1, 2, 3, 4, 5];
	let e = vec![0x20, 0x01, 0x23, 0x45];
	let h = hex_prefix_encode(&v, true);
	assert_eq!(h, e);

	let v = vec![1, 2, 3, 4, 5];
	let e = vec![0x31, 0x23, 0x45];
	let h = hex_prefix_encode(&v, true);
	assert_eq!(h, e);

	let v = vec![1, 2, 3, 4];
	let e = vec![0x00, 0x12, 0x34];
	let h = hex_prefix_encode(&v, false);
	assert_eq!(h, e);

	let v = vec![4, 1];
	let e = vec![0x20, 0x41];
	let h = hex_prefix_encode(&v, true);
	assert_eq!(h, e);
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use hash::H256;
	use super::trie_root;

	#[test]
	fn simple_test() {
		assert_eq!(trie_root(vec![
			(b"A".to_vec(), b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_vec())
		]), H256::from_str("d23786fb4a010da3ce639d66d5e904a11dbc02746d1ce25029e53290cabf28ab").unwrap());
	}

	#[test]
	fn test_triehash_out_of_order() {
		assert!(trie_root(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
			(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
		]) ==
		trie_root(vec![
			(vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
			(vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
			(vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
		]));
	}

}
