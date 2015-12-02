//! Generete trie root.
//! 
//! This module should be used to generate trie root hash.

use std::collections::BTreeMap;
use std::cmp;
use hash::*;
use sha3::*;
use rlp;
use rlp::RlpStream;
use vector::SharedPrefix;

// todo: verify if example for ordered_trie_root is valid
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
pub fn ordered_trie_root(input: Vec<Vec<u8>>) -> H256 {
	let gen_input = input
		// first put elements into btree to sort them by nibbles
		// optimize it later
		.into_iter()
		.fold(BTreeMap::new(), | mut acc, vec | {
			let len = acc.len();
			acc.insert(rlp::encode(&len), vec);
			acc
		})
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
pub fn trie_root(input: Vec<(Vec<u8>, Vec<u8>)>) -> H256 {
	let gen_input = input
		// first put elements into btree to sort them and to remove duplicates
		.into_iter()
		.fold(BTreeMap::new(), | mut acc, (k, v) | {
			acc.insert(k, v);
			acc
		})
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

	//println!("input: {:?}", input); 
	// in case of empty slice, just append empty data
	if inlen == 0 {
		stream.append_empty_data();
		return;
	}

	// take slices
	let key: &Vec<u8> = &input[0].0;
	let value: &[u8] = &input[0].1;

	// if the slice contains just one item, append the suffix of the key
	// and then append value
	if inlen == 1 {
		stream.append_list(2);
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
			cmp::min(key.shared_prefix_len(&k), acc)
		});

//	println!("shared_prefix: {}, prefix_len: {}", shared_prefix, pre_len);
	// if shared prefix is higher than current prefix append its
	// new part of the key to the stream
	// then recursively append suffixes of all items who had this key
	if shared_prefix > pre_len {
		stream.append_list(2);
		stream.append(&hex_prefix_encode(&key[pre_len..shared_prefix], false));
		hash256aux(input, shared_prefix, stream);
		return;
	}

	// an item for every possible nibble/suffix
	// + 1 for data 
	stream.append_list(17);

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
				.take_while(| pair | { /*println!("{:?}", pair.0);*/ pair.0[pre_len] == i }).count(), 
				//.take_while(|&q| q == i).count(),
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

	// if fist key len is equal prefix, append it's value
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
	use rustc_serialize::hex::FromHex;
	use hash::*;
	use triehash::*;

	#[test]
	fn empty_trie_root() {
		assert_eq!(trie_root(vec![]), H256::from_str("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").unwrap());
	}

	#[test]
	fn single_trie_item() {
		let v = vec![(From::from("A"), From::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))];
		assert_eq!(trie_root(v), H256::from_str("d23786fb4a010da3ce639d66d5e904a11dbc02746d1ce25029e53290cabf28ab").unwrap());
	}

	#[test]
	fn foo_trie_item() {

		let v = vec![
			(From::from("foo"), From::from("bar")),
			(From::from("food"), From::from("bass"))
		];
		
		assert_eq!(trie_root(v), H256::from_str("17beaa1648bafa633cda809c90c04af50fc8aed3cb40d16efbddee6fdf63c4c3").unwrap());
	}

	#[test]
	fn dogs_trie_item() {

		let v = vec![
			(From::from("doe"), From::from("reindeer")),
			(From::from("dog"), From::from("puppy")),
			(From::from("dogglesworth"), From::from("cat")),
		];
		
		assert_eq!(trie_root(v), H256::from_str("8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3").unwrap());
	}

	#[test]
	fn puppy_trie_items() {

		let v = vec![
			(From::from("do"), From::from("verb")),
			(From::from("dog"), From::from("puppy")),
			(From::from("doge"), From::from("coin")),
			(From::from("horse"), From::from("stallion")),
		];
		
		assert_eq!(trie_root(v), H256::from_str("5991bb8c6514148a29db676a14ac506cd2cd5775ace63c30a4fe457715e9ac84").unwrap());
	}

	#[test]
	fn out_of_order() {
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

	#[test]
	fn test_trie_root() {
		let v = vec![
		
			("0000000000000000000000000000000000000000000000000000000000000045".from_hex().unwrap(), 
			 "22b224a1420a802ab51d326e29fa98e34c4f24ea".from_hex().unwrap()),

			("0000000000000000000000000000000000000000000000000000000000000046".from_hex().unwrap(),
			 "67706c2076330000000000000000000000000000000000000000000000000000".from_hex().unwrap()),

			("000000000000000000000000697c7b8c961b56f675d570498424ac8de1a918f6".from_hex().unwrap(),
			 "6f6f6f6820736f2067726561742c207265616c6c6c793f000000000000000000".from_hex().unwrap()),

			("0000000000000000000000007ef9e639e2733cb34e4dfc576d4b23f72db776b2".from_hex().unwrap(),
			 "4655474156000000000000000000000000000000000000000000000000000000".from_hex().unwrap()),

			("000000000000000000000000ec4f34c97e43fbb2816cfd95e388353c7181dab1".from_hex().unwrap(),
			 "4e616d6552656700000000000000000000000000000000000000000000000000".from_hex().unwrap()),

			("4655474156000000000000000000000000000000000000000000000000000000".from_hex().unwrap(),
			 "7ef9e639e2733cb34e4dfc576d4b23f72db776b2".from_hex().unwrap()),

			("4e616d6552656700000000000000000000000000000000000000000000000000".from_hex().unwrap(),
			 "ec4f34c97e43fbb2816cfd95e388353c7181dab1".from_hex().unwrap()),

			("6f6f6f6820736f2067726561742c207265616c6c6c793f000000000000000000".from_hex().unwrap(),
			 "697c7b8c961b56f675d570498424ac8de1a918f6".from_hex().unwrap())

		];

		assert_eq!(trie_root(v), H256::from_str("9f6221ebb8efe7cff60a716ecb886e67dd042014be444669f0159d8e68b42100").unwrap());
	}
}
