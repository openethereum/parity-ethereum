// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use std::collections::HashMap;
use sha3::SHA3_EMPTY;
use rlp::{UntrustedRlp, View, Compressible, SHA3_NULL_RLP, encode, ElasticArray1024, Stream, RlpStream};

/// Stores RLPs used for compression
struct InvalidRlpSwapper {
	invalid_to_valid: HashMap<Vec<u8>, Vec<u8>>,
	valid_to_invalid: HashMap<Vec<u8>, Vec<u8>>,
}

impl InvalidRlpSwapper {
	/// Construct a swapper from a list of common RLPs
	fn new(rlps_to_swap: Vec<Vec<u8>>) -> Self {
		if rlps_to_swap.len() > 0x80 {
			panic!("Invalid usage, only 128 RLPs can be swappable.");
		}
		let mut invalid_to_valid = HashMap::new();
		let mut valid_to_invalid = HashMap::new();
		for (i, rlp) in rlps_to_swap.iter().enumerate() {
			let invalid = vec!(0x81, i as u8); 	
			invalid_to_valid.insert(invalid.clone(), rlp.clone());
			valid_to_invalid.insert(rlp.to_owned(), invalid);
		}
		InvalidRlpSwapper {
			invalid_to_valid: invalid_to_valid,
			valid_to_invalid: valid_to_invalid
		}
	}
	/// Get a valid RLP corresponding to an invalid one
	fn get_valid(&self, invalid_rlp: &[u8]) -> Option<&[u8]> {
		self.invalid_to_valid.get(invalid_rlp).map(|v| v.as_slice())
	}
	/// Get an invalid RLP corresponding to a valid one
	fn get_invalid(&self, valid_rlp: &[u8]) -> Option<&[u8]> {
		self.valid_to_invalid.get(valid_rlp).map(|v| v.as_slice())
	}
}

#[test]
fn invalid_rlp_swapper() {
	let to_swap = vec![vec![0x83, b'c', b'a', b't'], vec![0x83, b'd', b'o', b'g']];
	let swapper = InvalidRlpSwapper::new(to_swap);
	let invalid_rlp = vec![vec![0x81, 0x00], vec![0x81, 0x01]];
	assert_eq!(Some(invalid_rlp[0].as_slice()), swapper.get_invalid(&[0x83, b'c', b'a', b't']));
	assert_eq!(None, swapper.get_invalid(&[0x83, b'b', b'a', b't']));
	assert_eq!(Some(vec![0x83, b'd', b'o', b'g'].as_slice()), swapper.get_valid(&invalid_rlp[1]));
}

include!("commonrlps.rs");

fn to_elastic(slice: &[u8]) -> ElasticArray1024<u8> {
	let mut out = ElasticArray1024::new();
	out.append_slice(slice);
	out
}

fn map_rlp<F>(rlp: &UntrustedRlp, f: F) -> Option<ElasticArray1024<u8>> where
	F: Fn(&UntrustedRlp) -> Option<ElasticArray1024<u8>> {
	match rlp.iter()
  .fold((false, RlpStream::new_list(rlp.item_count())),
  |(is_some, mut acc), subrlp| {
  	let new = f(&subrlp);
  	if let Some(ref insert) = new {
  		acc.append_raw(&insert[..], 1);
  	} else {
  		acc.append_raw(subrlp.as_raw(), 1);
  	}
  	(is_some || new.is_some(), acc)
  }) {
  	(true, s) => Some(s.drain()),
  	_ => None,
  }
}

impl<'a> Compressible for UntrustedRlp<'a> {
	fn simple_compress(&self) -> ElasticArray1024<u8> {
		if self.is_data() {
			to_elastic(INVALID_RLP_SWAPPER.get_invalid(self.as_raw()).unwrap_or(self.as_raw()))
		} else {
			map_rlp(self, |rlp| Some(rlp.simple_compress())).unwrap_or(to_elastic(self.as_raw()))
		}
	}

	fn simple_decompress(&self) -> ElasticArray1024<u8> {
		if self.is_data() {
			to_elastic(INVALID_RLP_SWAPPER.get_valid(self.as_raw()).unwrap_or(self.as_raw()))
		} else {
			map_rlp(self, |rlp| Some(rlp.simple_decompress())).unwrap_or(to_elastic(self.as_raw()))
		}
	}

	fn compress(&self) -> Option<ElasticArray1024<u8>> {
		let simple_swap = ||
			INVALID_RLP_SWAPPER.get_invalid(self.as_raw()).map(|b| to_elastic(&b));
		if self.is_data() {
			// Try to treat the inside as RLP.
			return match self.payload_info() {
				// Shortest decompressed account is 70, so simply try to swap the value.
				Ok(ref p) if p.value_len < 70 => simple_swap(),
				_ => {
					if let Ok(d) = self.data() {
						if let Some(new_d) = UntrustedRlp::new(&d).compress() {
							// If compressed put in a special list, with first element being invalid code.
							let mut rlp = RlpStream::new_list(2);
							rlp.append_raw(&[0x81, 0xcc], 1);
							rlp.append_raw(&new_d[..], 1);
							return Some(rlp.drain());
						}
					}
					simple_swap()
				},
			};
		}
		// Iterate through RLP while checking if it has been compressed.
		map_rlp(self, |rlp| rlp.compress())
	}

	fn decompress(&self) -> Option<ElasticArray1024<u8>> {
		let simple_swap = ||
			INVALID_RLP_SWAPPER.get_valid(self.as_raw()).map(|b| to_elastic(&b));
		// Simply decompress data.
		if self.is_data() { return simple_swap(); }
		match self.item_count() {
			// Look for special compressed list, which contains nested data.
			2 if self.at(0).map(|r| r.as_raw() == &[0x81, 0xcc]).unwrap_or(false) =>
				self.at(1).ok().map_or(simple_swap(),
				|r| r.decompress().map(|d| { let v = d.to_vec(); encode(&v) })),
			// Iterate through RLP while checking if it has been compressed.
			_ => map_rlp(self, |rlp| rlp.decompress()),
  	}
	}
}

#[cfg(test)]
mod tests {
	use rlp::{UntrustedRlp, Compressible, View};

	#[test]
	fn compressible() {
		let nested_basic_account_rlp = vec![184, 70, 248, 68, 4, 2, 160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 160, 197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112];
		let nested_rlp = UntrustedRlp::new(&nested_basic_account_rlp);
		let compressed = nested_rlp.compress().unwrap().to_vec();
		assert_eq!(compressed, vec![201, 129, 204, 198, 4, 2, 129, 0, 129, 1]);
		let compressed_rlp = UntrustedRlp::new(&compressed);
		assert_eq!(compressed_rlp.decompress().unwrap().to_vec(), nested_basic_account_rlp);
	}

	#[test]
	fn malformed_rlp() {
		let malformed = vec![248, 81, 128, 128, 128, 128, 128, 160, 12, 51, 241, 93, 69, 218, 74, 138, 79, 115, 227, 44, 216, 81, 46, 132, 85, 235, 96, 45, 252, 48, 181, 29, 75, 141, 217, 215, 86, 160, 109, 130, 160, 140, 36, 93, 200, 109, 215, 100, 241, 246, 99, 135, 92, 168, 149, 170, 114, 9, 143, 4, 93, 25, 76, 54, 176, 119, 230, 170, 154, 105, 47, 121, 10, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128];
		let malformed_rlp = UntrustedRlp::new(&malformed);
		assert!(malformed_rlp.decompress().is_none());
	}
	
	#[test]
	#[ignore]
	#[allow(dead_code)]
	fn analyze_db() {
		use std::collections::HashMap;
		use kvdb::*;
		let path = "/home/keorn/.parity/906a34e69aec8c0d/v5.3-sec-overlayrecent/state".to_string();
		let values: Vec<_> = Database::open_default(&path).unwrap().iter().map(|(_, v)| v).collect();
		let mut rlp_counts: HashMap<_, u32> = HashMap::new();
		let mut rlp_sizes: HashMap<_, u32> = HashMap::new();

		fn flat_rlp<'a>(acc: &mut Vec<UntrustedRlp<'a>>, rlp: UntrustedRlp<'a>) {
			match rlp.is_data() {
				true => if rlp.size()>=70 {
					match rlp.data() {
						Ok(x) => flat_rlp(acc, UntrustedRlp::new(x)),
						_ => acc.push(rlp),
					}
				} else {
					acc.push(rlp);
				},
				false => for r in rlp.iter() { flat_rlp(acc, r); },
			}
		}

		fn space_saving(bytes: &[u8]) -> u32 {
			let l = bytes.len() as u32;
			match l >= 2 {
				true => l-2,
				false => 0,
			}
		}

		fn is_account<'a>(rlp: &UntrustedRlp<'a>) -> bool {
			rlp.is_list() && (rlp.item_count() == 4)
		}

		for v in values.iter() {
			let rlp = UntrustedRlp::new(&v);
			let mut flat = Vec::new();
			flat_rlp(&mut flat, rlp);
			for r in flat.iter() {
				*rlp_counts.entry(r.as_raw()).or_insert(0) += 1;
				//let replacement = r.compress().to_vec();
				*rlp_sizes.entry(r.as_raw()).or_insert(0) += space_saving(r.as_raw());
			}
		}
		let mut size_vec: Vec<_> = rlp_sizes.iter().collect();
		size_vec.sort_by(|a, b| b.1.cmp(a.1));

		for v in size_vec.iter().filter(|v| rlp_counts.get(v.0).unwrap()>&100).take(20) {
			println!("{:?}, {:?}", v, rlp_counts.get(v.0).unwrap());
		}
		println!("DONE");
	}

	#[test]
	#[ignore]
	fn test_compression() {
		use kvdb::*;
		let path = "/home/keorn/.parity/906a34e69aec8c0d/v5.3-sec-overlayrecent/state".to_string();
		let values: Vec<_> = Database::open_default(&path).unwrap().iter().map(|(_, v)| v).collect();
		let mut init_size = 0;
		let mut comp_size = 0;

		for v in values.iter() {
			init_size += v.len();
			let rlp = UntrustedRlp::new(&v);
			let compressed = rlp.compress().map(|b| b.to_vec()).unwrap_or(v.to_vec());
			comp_size += compressed.len();
			//assert_eq!(UntrustedRlp::new(&compressed.as_slice()).decompress().map(|b| b.to_vec()).unwrap_or(v.to_vec()), v.to_vec());
		}
		println!("Initial bytes {:?}, compressed bytes: {:?}", init_size, comp_size);
		assert!(init_size > comp_size);
	}
}
