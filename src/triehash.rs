//! Generete trie root

use std::collections::BTreeMap;
use std::cmp;
use hash::*;
use sha3::*;
use rlp;
use rlp::RlpStream;

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
///  
/// ```rust
///	extern crate ethcore_util as util;
///	use util::triehash::*;
///  
///	fn main() {
///		let v = vec![0, 0, 1, 2, 3, 4, 5];
///		let e = vec![0x10, 0x01, 0x23, 0x45];
///		let h = hex_prefix_encode(&v, false);
///		assert_eq!(h, e);
///		
///		let v = vec![0, 1, 2, 3, 4, 5];
///		let e = vec![0x00, 0x01, 0x23, 0x45];
///		let h = hex_prefix_encode(&v, false);
///		assert_eq!(h, e);
///		
///		let v = vec![0, 1, 2, 3, 4, 5];
///		let e = vec![0x20, 0x01, 0x23, 0x45];
///		let h = hex_prefix_encode(&v, true);
///		assert_eq!(h, e);
///		
///		let v = vec![1, 2, 3, 4, 5];
///		let e = vec![0x31, 0x23, 0x45];
///		let h = hex_prefix_encode(&v, true);
///		assert_eq!(h, e);
///		
///		let v = vec![1, 2, 3, 4];
///		let e = vec![0x00, 0x12, 0x34];
///		let h = hex_prefix_encode(&v, false);
///		assert_eq!(h, e);
///	}
/// ```
///  
pub fn hex_prefix_encode(nibbles: &[u8], leaf: bool) -> Vec<u8> {
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
/// 
/// ```rust
///	extern crate ethcore_util as util;
///	use util::triehash::*;
///
///	fn main () {
///		let v = vec![0x31, 0x23, 0x45];
///		let e = vec![3, 1, 2, 3, 4, 5];
///		assert_eq!(as_nibbles(&v), e);
///	}
/// ```
pub fn as_nibbles(bytes: &[u8]) -> Vec<u8> {
	let mut res = vec![];
	res.reserve(bytes.len() * 2);
	for i in 0..bytes.len() {
		res.push(bytes[i] >> 4);
		res.push((bytes[i] << 4) >> 4);
	}
	res
}

struct NibblePair {
	nibble: Vec<u8>,
	data: Vec<u8>
}

pub fn ordered_trie_root(data: Vec<Vec<u8>>) -> H256 {
	let vec: Vec<NibblePair> = data
		// first put elements into btree to sort them by nibbles
		// optimize it later
		.into_iter()
		.fold(BTreeMap::new(), | mut acc, vec | {
			let len = acc.len();
			acc.insert(as_nibbles(&rlp::encode(&len)), vec);
			acc
		})
		// then move them to a vector
		.into_iter()
		.map(|(k, v)| NibblePair { nibble: k, data: v } )
		.collect();

	let out = match vec.len() {
		0 => rlp::encode(&""),
		_ => {
			let mut stream = RlpStream::new();
			hash256rlp(&vec, 0, &mut stream);
			stream.out().unwrap()
		}
	};
	
	out.sha3()
}

fn shared_prefix_length<T>(v1: &[T], v2: &[T]) -> usize where T: Eq {
	let len = cmp::min(v1.len(), v2.len());
	(0..len).take_while(|&i| v1[i] == v2[i]).count()
}

fn hash256rlp(vec: &[NibblePair], pre_len: usize, stream: &mut RlpStream) {
	match vec.len() {
		0 => stream.append(&""),
		1 => stream.append_list(2).append(&hex_prefix_encode(&vec[0].nibble, true)).append(&vec[0].data),
		_ => {
			let shared_prefix = vec.iter()
				// skip first element
				.skip(1)
				// get minimum number of shared nibbles between first string and each successive
				.fold(usize::max_value(), | acc, pair | cmp::min(shared_prefix_length(&vec[0].nibble, &pair.nibble), acc) );
			//match shared_prefix > pre_len {

				//true => hex_prefix_encode(&vec[0].nibble
			//}
			panic!();
			
		}
	};
}

#[cfg(test)]
mod tests {
}
