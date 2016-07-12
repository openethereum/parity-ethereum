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
use rlp::untrusted_rlp::BasicDecoder;
use rlp::{UntrustedRlp, View, PayloadInfo, DecoderError, Decoder, Compressible, SHA3_NULL_RLP, encode, ElasticArray1024, Stream, RlpStream};

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

lazy_static! {
	/// Swapper with common long RLPs, up to 128 can be added.
	static ref INVALID_RLP_SWAPPER: InvalidRlpSwapper = InvalidRlpSwapper::new(vec![encode(&SHA3_NULL_RLP).to_vec(), encode(&SHA3_EMPTY).to_vec()]);
}

fn slice_to_elastic(rlp: &[u8]) -> ElasticArray1024<u8> {
	let mut result = ElasticArray1024::new();
	result.append_slice(rlp);
	result
}


impl<'a> Compressible for UntrustedRlp<'a> {
	fn swap<F>(&self, swapper: &F) -> ElasticArray1024<u8>
	where F: Fn(&[u8]) -> Option<&[u8]> {
		let raw = self.as_raw();
		slice_to_elastic(swapper(raw).unwrap_or(raw))
	}

	fn swap_all<F>(&self, swapper: &F, account_size: usize) -> ElasticArray1024<u8>
	where F: Fn(&[u8]) -> Option<&[u8]> {
		println!("{:?}", self.as_raw());
		if self.is_data() {
			match self.payload_info() {
				// If short simply try to replace the value.
				Ok(ref p) if p.value_len < account_size => self.swap(swapper),
				// If long and has data, try to treat the inside as RLP.
				_ => self.data().ok().map_or(self.swap(swapper), |d| {
					let new_d = UntrustedRlp::new(d).swap_all(swapper, account_size).to_vec();
					if &new_d == &d {
						// Just return if no compression was achieved.
						slice_to_elastic(self.as_raw())
					} else {
						// Attach correct prefix if compressed.
						let out = encode(&new_d);
						//println!("Entry: {:?}", self.as_raw());
						//println!("d1: {:?}, d2: {:?}", self.data().unwrap(), new_d);
						//assert_eq!(&out[..], self.as_raw());
						out
					}
				},)
			}
		} else {
			// Try to iterate if it might be a list.
			match self.item_count() {
				0 => self.swap(swapper),
				c => {
  				let mut rlp = RlpStream::new_list(c);
  				let mut is_changed = false;
  				for subrlp in self.iter() {
						//println!("Loop: {:?}", subrlp.as_raw());
  					let new = subrlp.swap_all(swapper, account_size);
						rlp.append_raw(&new, 1);
					}
					rlp.drain()
				},
  		}
  	}
	}

	fn compress(&self) -> ElasticArray1024<u8> {
		// Shortest decompressed account is 70.
		self.swap_all(&|b| INVALID_RLP_SWAPPER.get_invalid(b), 70)
	}

	fn decompress(&self) -> ElasticArray1024<u8> {
		// Shortest compressed account is 7.
		self.swap_all(&|b| INVALID_RLP_SWAPPER.get_valid(b), 7)
	}
}

struct DecompressingDecoder<'a> {
	rlp: UntrustedRlp<'a>,
	swapper: &'a INVALID_RLP_SWAPPER,
}

impl<'a> DecompressingDecoder<'a> {
	pub fn new(rlp: UntrustedRlp<'a>) -> DecompressingDecoder<'a> {
		DecompressingDecoder {
			rlp: rlp,
			swapper: &INVALID_RLP_SWAPPER,
		}
	}

	/// Return first item info.
	fn payload_info(bytes: &[u8]) -> Result<PayloadInfo, DecoderError> {
		let item = try!(PayloadInfo::from(bytes));
		match item.header_len.checked_add(item.value_len) { 
			Some(x) if x <= bytes.len() => Ok(item), 
			_ => Err(DecoderError::RlpIsTooShort), 
		}
	}
}

impl<'a> Decoder for DecompressingDecoder<'a> {
	fn read_value<T, F>(&self, f: &F) -> Result<T, DecoderError>
	where F: Fn(&[u8]) -> Result<T, DecoderError> {
		match BasicDecoder::new(self.rlp.clone()).read_value(f) {
			// Try again with decompression.
			Err(DecoderError::RlpInvalidIndirection) => {
				let decompressed = self.rlp.decompress();
				BasicDecoder::new(UntrustedRlp::new(&decompressed)).read_value(f)
			},
			// Just return on valid RLP.
			x => x,
		}
	}

	fn as_raw(&self) -> &[u8] {
		self.rlp.as_raw()
	}

	fn as_rlp(&self) -> &UntrustedRlp {
		&self.rlp
	}
}

#[test]
fn invalid_rlp_swapper() {
	let swapper = InvalidRlpSwapper::new(vec![vec![0x83, b'c', b'a', b't'], vec![0x83, b'd', b'o', b'g']]);
	let invalid_rlp = vec![vec![0x81, 0x00], vec![0x81, 0x01]];
	assert_eq!(Some(invalid_rlp[0].as_slice()), swapper.get_invalid(&[0x83, b'c', b'a', b't']));
	assert_eq!(None, swapper.get_invalid(&[0x83, b'b', b'a', b't']));
	assert_eq!(Some(vec![0x83, b'd', b'o', b'g'].as_slice()), swapper.get_valid(&invalid_rlp[1]));
}

#[test]
fn compressible() {
	let nested_basic_account_rlp = vec![184, 70, 248, 68, 4, 2, 160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 160, 197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112];
	let nested_rlp = UntrustedRlp::new(&nested_basic_account_rlp);
	let compressed = nested_rlp.compress().to_vec();
	assert_eq!(compressed, vec![135, 198, 4, 2, 129, 0, 129, 1]);
	let compressed_rlp = UntrustedRlp::new(&compressed);
	assert_eq!(compressed_rlp.decompress().to_vec(), nested_basic_account_rlp);
}

#[test]
/// Fails when not checking for empty RLP iterator.
fn malformed_rlp_one() {
	let malformed = vec![197, 165, 153, 36, 204, 227, 139, 156, 139, 239, 24, 223, 247, 53, 105, 160, 226, 251, 141, 8];
	let malformed_rlp = UntrustedRlp::new(&malformed);
	assert_eq!(malformed_rlp.decompress().to_vec(), malformed);
}

#[test]
/// Fails when trying to encode uncompressed malformed RLP.
fn malformed_rlp_two() {
	let malformed = vec![248, 81, 128, 128, 128, 128, 128, 160, 12, 51, 241, 93, 69, 218, 74, 138, 79, 115, 227, 44, 216, 81, 46, 132, 85, 235, 96, 45, 252, 48, 181, 29, 75, 141, 217, 215, 86, 160, 109, 130, 160, 140, 36, 93, 200, 109, 215, 100, 241, 246, 99, 135, 92, 168, 149, 170, 114, 9, 143, 4, 93, 25, 76, 54, 176, 119, 230, 170, 154, 105, 47, 121, 10, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128];
	let malformed_rlp = UntrustedRlp::new(&malformed);
	assert_eq!(malformed_rlp.decompress().to_vec(), malformed);
}

#[test]
/// Fails when trying to decode malformed short list RLP.
fn malformed_rlp_three() {
	let malformed = vec![160, 213, 61, 174, 16, 148, 93, 60, 68, 40, 61, 3, 247, 17, 184, 93, 133, 138, 190, 84, 180, 47, 110, 18, 137, 201, 163, 234, 7, 98, 131, 242, 47];
	let malformed_rlp = UntrustedRlp::new(&malformed);
	assert_eq!(malformed_rlp.decompress().to_vec(), malformed);
}

#[test]
/// Fails when trying to decode malformed short list RLP.
fn weird() {
	let malformed = vec![245, 158, 53, 252, 25, 189, 112, 31, 37, 119, 153, 221, 219, 115, 64, 123, 18, 55, 19, 76, 136, 82, 90, 59, 154, 236, 197, 32, 169, 71, 1, 254, 149, 148, 209, 71, 98, 123, 94, 19, 201, 123, 6, 212, 69, 233, 0, 38, 86, 4, 28, 194, 75, 150];
	let malformed_rlp = UntrustedRlp::new(&malformed);
	assert_eq!(malformed_rlp.decompress().to_vec(), malformed);
}

#[test]
fn decompressing_decoder() {
	use rustc_serialize::hex::ToHex;
	let basic_account_rlp = vec![248, 68, 4, 2, 160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 160, 197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112];
	let rlp = UntrustedRlp::new(&basic_account_rlp);
	let compressed = rlp.compress().to_vec();
	assert_eq!(compressed, vec![198, 4, 2, 129, 0, 129, 1]);
	let compressed_rlp = UntrustedRlp::new(&compressed);
	let f = | b: &[u8] | Ok(b.to_vec());
	let decoded: Vec<_> = compressed_rlp.iter().map(|v| DecompressingDecoder::new(v).read_value(&f).expect("")).collect();
	assert_eq!(decoded[0], vec![4]);
	assert_eq!(decoded[1], vec![2]);
	assert_eq!(decoded[2].to_hex(), SHA3_NULL_RLP.hex());
	assert_eq!(decoded[3].to_hex(), SHA3_EMPTY.hex());
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
		let compressed = rlp.compress().to_vec();
		comp_size += compressed.len();
		assert_eq!(UntrustedRlp::new(&compressed.as_slice()).decompress().to_vec(), v.to_vec());
	}
	println!("Initial bytes {:?}, compressed bytes: {:?}", init_size, comp_size);
	assert!(init_size > comp_size);
}
