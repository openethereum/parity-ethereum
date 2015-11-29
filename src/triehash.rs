//! Generete trie root

//use std::collections::HashMap;
//use hash::*;
//use rlp;

///
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
pub fn hex_prefix_encode(hex: &[u8], leaf: bool) -> Vec<u8> {
	let inlen = hex.len();
	let oddness_factor = inlen % 2;
	// next even number divided by two
	let reslen = (inlen + 2) >> 1;
	let mut res = vec![];
	res.reserve(reslen);

	let first_byte = {
		let mut bits = ((inlen as u8 & 1) + (2 * leaf as u8)) << 4;
		if oddness_factor == 1 {
			bits += hex[0];
		}
		bits
	};

	res.push(first_byte);

	let mut offset = oddness_factor;	
	while offset < inlen {
		let byte = (hex[offset] << 4) + hex[offset + 1];
		res.push(byte);
		offset += 2;
	}

	res
}

#[cfg(test)]
mod tests {
}
