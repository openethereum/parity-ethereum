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

use std::{fmt, cmp};
use std::str::FromStr;
use ::{Encodable, RlpDecodable, UntrustedRlp, RlpStream, View, Stream, DecoderError};
use bigint::uint::U256;

#[test]
fn rlp_at() {
	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	{
		let rlp = UntrustedRlp::new(&data);
		assert!(rlp.is_list());
		let animals: Vec<String> = rlp.as_val().unwrap();
		assert_eq!(animals, vec!["cat".to_owned(), "dog".to_owned()]);

		let cat = rlp.at(0).unwrap();
		assert!(cat.is_data());
		assert_eq!(cat.as_raw(), &[0x83, b'c', b'a', b't']);
		assert_eq!(cat.as_val::<String>().unwrap(), "cat".to_owned());

		let dog = rlp.at(1).unwrap();
		assert!(dog.is_data());
		assert_eq!(dog.as_raw(), &[0x83, b'd', b'o', b'g']);
		assert_eq!(dog.as_val::<String>().unwrap(), "dog".to_owned());

		let cat_again = rlp.at(0).unwrap();
		assert!(cat_again.is_data());
		assert_eq!(cat_again.as_raw(), &[0x83, b'c', b'a', b't']);
		assert_eq!(cat_again.as_val::<String>().unwrap(), "cat".to_owned());
	}
}

#[test]
fn rlp_at_err() {
	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o'];
	{
		let rlp = UntrustedRlp::new(&data);
		assert!(rlp.is_list());

		let cat_err = rlp.at(0).unwrap_err();
		assert_eq!(cat_err, DecoderError::RlpIsTooShort);

		let dog_err = rlp.at(1).unwrap_err();
		assert_eq!(dog_err, DecoderError::RlpIsTooShort);
	}
}

#[test]
fn rlp_iter() {
	let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
	{
		let rlp = UntrustedRlp::new(&data);
		let mut iter = rlp.iter();

		let cat = iter.next().unwrap();
		assert!(cat.is_data());
		assert_eq!(cat.as_raw(), &[0x83, b'c', b'a', b't']);

		let dog = iter.next().unwrap();
		assert!(dog.is_data());
		assert_eq!(dog.as_raw(), &[0x83, b'd', b'o', b'g']);

		let none = iter.next();
		assert!(none.is_none());

		let cat_again = rlp.at(0).unwrap();
		assert!(cat_again.is_data());
		assert_eq!(cat_again.as_raw(), &[0x83, b'c', b'a', b't']);
	}
}

struct ETestPair<T>(T, Vec<u8>) where T: Encodable;

fn run_encode_tests<T>(tests: Vec<ETestPair<T>>)
	where T: Encodable
{
	for t in &tests {
		let res = super::encode(&t.0);
		assert_eq!(&res[..], &t.1[..]);
	}
}

#[test]
fn encode_u16() {
	let tests = vec![
		ETestPair(0u16, vec![0x80u8]),
		ETestPair(0x100, vec![0x82, 0x01, 0x00]),
		ETestPair(0xffff, vec![0x82, 0xff, 0xff]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u32() {
	let tests = vec![
		ETestPair(0u32, vec![0x80u8]),
		ETestPair(0x10000, vec![0x83, 0x01, 0x00, 0x00]),
		ETestPair(0xffffff, vec![0x83, 0xff, 0xff, 0xff]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u64() {
	let tests = vec![
		ETestPair(0u64, vec![0x80u8]),
		ETestPair(0x1000000, vec![0x84, 0x01, 0x00, 0x00, 0x00]),
		ETestPair(0xFFFFFFFF, vec![0x84, 0xff, 0xff, 0xff, 0xff]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_u256() {
	let tests = vec![ETestPair(U256::from(0u64), vec![0x80u8]),
					 ETestPair(U256::from(0x1000000u64), vec![0x84, 0x01, 0x00, 0x00, 0x00]),
					 ETestPair(U256::from(0xffffffffu64),
							   vec![0x84, 0xff, 0xff, 0xff, 0xff]),
					 ETestPair(U256::from_str("8090a0b0c0d0e0f00910203040506077000000000000\
											   000100000000000012f0")
								   .unwrap(),
							   vec![0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
									0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77, 0x00,
									0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
									0x00, 0x00, 0x00, 0x00, 0x12, 0xf0])];
	run_encode_tests(tests);
}

#[test]
fn encode_str() {
	let tests = vec![ETestPair("cat", vec![0x83, b'c', b'a', b't']),
					 ETestPair("dog", vec![0x83, b'd', b'o', b'g']),
					 ETestPair("Marek", vec![0x85, b'M', b'a', b'r', b'e', b'k']),
					 ETestPair("", vec![0x80]),
					 ETestPair("Lorem ipsum dolor sit amet, consectetur adipisicing elit",
							   vec![0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i',
									b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l', b'o',
									b'r', b' ', b's', b'i', b't', b' ', b'a', b'm', b'e',
									b't', b',', b' ', b'c', b'o', b'n', b's', b'e', b'c',
									b't', b'e', b't', b'u', b'r', b' ', b'a', b'd', b'i',
									b'p', b'i', b's', b'i', b'c', b'i', b'n', b'g', b' ',
									b'e', b'l', b'i', b't'])];
	run_encode_tests(tests);
}

#[test]
fn encode_address() {
	use bigint::hash::H160;

	let tests = vec![
		ETestPair(H160::from("ef2d6d194084c2de36e0dabfce45d046b37d1106"),
				  vec![0x94, 0xef, 0x2d, 0x6d, 0x19, 0x40, 0x84, 0xc2, 0xde,
							 0x36, 0xe0, 0xda, 0xbf, 0xce, 0x45, 0xd0, 0x46,
							 0xb3, 0x7d, 0x11, 0x06])
	];
	run_encode_tests(tests);
}

/// Vec<u8> (Bytes) is treated as a single value
#[test]
fn encode_vector_u8() {
	let tests = vec![
		ETestPair(vec![], vec![0x80]),
		ETestPair(vec![0u8], vec![0]),
		ETestPair(vec![0x15], vec![0x15]),
		ETestPair(vec![0x40, 0x00], vec![0x82, 0x40, 0x00]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_vector_u64() {
	let tests = vec![
		ETestPair(vec![], vec![0xc0]),
		ETestPair(vec![15u64], vec![0xc1, 0x0f]),
		ETestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
		ETestPair(vec![0xffffffff, 1, 2, 3, 7, 0xff], vec![0xcb, 0x84, 0xff, 0xff, 0xff, 0xff,  1, 2, 3, 7, 0x81, 0xff]),
	];
	run_encode_tests(tests);
}

#[test]
fn encode_vector_str() {
	let tests = vec![ETestPair(vec!["cat", "dog"],
							   vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'])];
	run_encode_tests(tests);
}

struct DTestPair<T>(T, Vec<u8>) where T: RlpDecodable + fmt::Debug + cmp::Eq;

fn run_decode_tests<T>(tests: Vec<DTestPair<T>>) where T: RlpDecodable + fmt::Debug + cmp::Eq {
	for t in &tests {
		let res: T = super::decode(&t.1);
		assert_eq!(res, t.0);
	}
}

/// Vec<u8> (Bytes) is treated as a single value
#[test]
fn decode_vector_u8() {
	let tests = vec![
		DTestPair(vec![], vec![0x80]),
		DTestPair(vec![0u8], vec![0]),
		DTestPair(vec![0x15], vec![0x15]),
		DTestPair(vec![0x40, 0x00], vec![0x82, 0x40, 0x00]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u8() {
	let tests = vec![
		DTestPair(0x0u8, vec![0x80]),
		DTestPair(0x77u8, vec![0x77]),
		DTestPair(0xccu8, vec![0x81, 0xcc]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u16() {
	let tests = vec![
		DTestPair(0x100u16, vec![0x82, 0x01, 0x00]),
		DTestPair(0xffffu16, vec![0x82, 0xff, 0xff]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u32() {
	let tests = vec![
		DTestPair(0x10000u32, vec![0x83, 0x01, 0x00, 0x00]),
		DTestPair(0xffffffu32, vec![0x83, 0xff, 0xff, 0xff]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u64() {
	let tests = vec![
		DTestPair(0x1000000u64, vec![0x84, 0x01, 0x00, 0x00, 0x00]),
		DTestPair(0xFFFFFFFFu64, vec![0x84, 0xff, 0xff, 0xff, 0xff]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_u256() {
	let tests = vec![DTestPair(U256::from(0u64), vec![0x80u8]),
					 DTestPair(U256::from(0x1000000u64), vec![0x84, 0x01, 0x00, 0x00, 0x00]),
					 DTestPair(U256::from(0xffffffffu64),
							   vec![0x84, 0xff, 0xff, 0xff, 0xff]),
					 DTestPair(U256::from_str("8090a0b0c0d0e0f00910203040506077000000000000\
											   000100000000000012f0")
								   .unwrap(),
							   vec![0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
									0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77, 0x00,
									0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
									0x00, 0x00, 0x00, 0x00, 0x12, 0xf0])];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_str() {
	let tests = vec![DTestPair("cat".to_owned(), vec![0x83, b'c', b'a', b't']),
					 DTestPair("dog".to_owned(), vec![0x83, b'd', b'o', b'g']),
					 DTestPair("Marek".to_owned(),
							   vec![0x85, b'M', b'a', b'r', b'e', b'k']),
					 DTestPair("".to_owned(), vec![0x80]),
					 DTestPair("Lorem ipsum dolor sit amet, consectetur adipisicing elit"
								   .to_owned(),
							   vec![0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i',
									b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l', b'o',
									b'r', b' ', b's', b'i', b't', b' ', b'a', b'm', b'e',
									b't', b',', b' ', b'c', b'o', b'n', b's', b'e', b'c',
									b't', b'e', b't', b'u', b'r', b' ', b'a', b'd', b'i',
									b'p', b'i', b's', b'i', b'c', b'i', b'n', b'g', b' ',
									b'e', b'l', b'i', b't'])];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_address() {
	use bigint::hash::H160;

	let tests = vec![
		DTestPair(H160::from("ef2d6d194084c2de36e0dabfce45d046b37d1106"),
				  vec![0x94, 0xef, 0x2d, 0x6d, 0x19, 0x40, 0x84, 0xc2, 0xde,
							 0x36, 0xe0, 0xda, 0xbf, 0xce, 0x45, 0xd0, 0x46,
							 0xb3, 0x7d, 0x11, 0x06])
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_vector_u64() {
	let tests = vec![
		DTestPair(vec![], vec![0xc0]),
		DTestPair(vec![15u64], vec![0xc1, 0x0f]),
		DTestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
		DTestPair(vec![0xffffffff, 1, 2, 3, 7, 0xff], vec![0xcb, 0x84, 0xff, 0xff, 0xff, 0xff,  1, 2, 3, 7, 0x81, 0xff]),
	];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_vector_str() {
	let tests = vec![DTestPair(vec!["cat".to_owned(), "dog".to_owned()],
							   vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'])];
	run_decode_tests(tests);
}

#[test]
fn decode_untrusted_vector_of_vectors_str() {
	let tests = vec![DTestPair(vec![vec!["cat".to_owned()]],
							   vec![0xc5, 0xc4, 0x83, b'c', b'a', b't'])];
	run_decode_tests(tests);
}

#[test]
fn test_decoding_array() {
	let v = vec![5u16, 2u16];
	let res = super::encode(&v);
	let arr: [u16; 2] = super::decode(&res);
	assert_eq!(arr[0], 5);
	assert_eq!(arr[1], 2);
}

#[test]
fn test_rlp_data_length_check()
{
	let data = vec![0x84, b'c', b'a', b't'];
	let rlp = UntrustedRlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert_eq!(Err(DecoderError::RlpInconsistentLengthAndData), as_val);
}

#[test]
fn test_rlp_long_data_length_check()
{
	let mut data: Vec<u8> = vec![0xb8, 255];
	for _ in 0..253 {
		data.push(b'c');
	}

	let rlp = UntrustedRlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert_eq!(Err(DecoderError::RlpInconsistentLengthAndData), as_val);
}

#[test]
fn test_the_exact_long_string()
{
	let mut data: Vec<u8> = vec![0xb8, 255];
	for _ in 0..255 {
		data.push(b'c');
	}

	let rlp = UntrustedRlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert!(as_val.is_ok());
}

#[test]
fn test_rlp_2bytes_data_length_check()
{
	let mut data: Vec<u8> = vec![0xb9, 2, 255]; // 512+255
	for _ in 0..700 {
		data.push(b'c');
	}

	let rlp = UntrustedRlp::new(&data);

	let as_val: Result<String, DecoderError> = rlp.as_val();
	assert_eq!(Err(DecoderError::RlpInconsistentLengthAndData), as_val);
}

#[test]
fn test_rlp_nested_empty_list_encode() {
	let mut stream = RlpStream::new_list(2);
	stream.append(&(Vec::new() as Vec<u32>));
	stream.append(&40u32);
	assert_eq!(stream.drain()[..], [0xc2u8, 0xc0u8, 40u8][..]);
}

#[test]
fn test_rlp_list_length_overflow() {
	let data: Vec<u8> = vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00];
	let rlp = UntrustedRlp::new(&data);
	let as_val: Result<String, DecoderError> = rlp.val_at(0);
	assert_eq!(Err(DecoderError::RlpIsTooShort), as_val);
}
