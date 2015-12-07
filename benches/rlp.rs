//! benchmarking for rlp
//! should be started with:
//! ```bash
//! multirust run nightly cargo bench
//! ```

#![feature(test)]

extern crate test;
extern crate ethcore_util;

use test::Bencher;
use std::str::FromStr;
use ethcore_util::rlp::{RlpStream, Rlp, Decodable};
use ethcore_util::uint::U256;

#[bench]
fn bench_stream_u64_value(b: &mut Bencher) {
	b.iter(|| {
		// u64
		let mut stream = RlpStream::new();
		stream.append(&0x1023456789abcdefu64);
		let _ = stream.out();
	});
}

#[bench]
fn bench_decode_u64_value(b: &mut Bencher) {
	b.iter(|| {
		// u64
		let data = vec![0x88, 0x10, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
		let rlp = Rlp::new(&data);
		let _ = u64::decode(&rlp);
	});
}

#[bench]
fn bench_stream_u256_value(b: &mut Bencher) {
	b.iter(|| {
		// u256
		let mut stream = RlpStream::new();
		stream.append(&U256::from_str("8090a0b0c0d0e0f009102030405060770000000000000001000000000\
		                               00012f0")
			               .unwrap());
		let _ = stream.out();
	});
}

#[bench]
fn bench_decode_u256_value(b: &mut Bencher) {
	b.iter(|| {
		// u256
		let data = vec![0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x09, 0x10, 0x20,
		                0x30, 0x40, 0x50, 0x60, 0x77, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0xf0];
		let rlp = Rlp::new(&data);
		let _ = U256::decode(&rlp);
	});
}

#[bench]
fn bench_stream_nested_empty_lists(b: &mut Bencher) {
	b.iter(|| {
		// [ [], [[]], [ [], [[]] ] ]
		let mut stream = RlpStream::new_list(3);
		stream.append_list(0);
		stream.append_list(1).append_list(0);
		stream.append_list(2).append_list(0).append_list(1).append_list(0);
		let _ = stream.out();
	});
}

#[bench]
fn bench_decode_nested_empty_lists(b: &mut Bencher) {
	b.iter(|| {
		// [ [], [[]], [ [], [[]] ] ]
		let data = vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0];
		let rlp = Rlp::new(&data);
		let _v0: Vec<u16> = Decodable::decode(&rlp.at(0));
		let _v1: Vec<Vec<u16>> = Decodable::decode(&rlp.at(1));
		let nested_rlp = rlp.at(2);
		let _v2a: Vec<u16> = Decodable::decode(&nested_rlp.at(0));
		let _v2b: Vec<Vec<u16>> = Decodable::decode(&nested_rlp.at(1));
	});
}

#[bench]
fn bench_stream_1000_empty_lists(b: &mut Bencher) {
	b.iter(|| {
		let mut stream = RlpStream::new_list(1000);
		for _ in 0..1000 {
			stream.append_list(0);
		}
		//let _ = stream.out();
	});
}
