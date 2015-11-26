//! benchmarking for rlp
//! should be started with:
//! ```bash
//! multirust run nightly cargo bench 
//! ```

#![feature(test)]

extern crate test;
extern crate ethcore_util;

use test::Bencher;
use ethcore_util::rlp;
use ethcore_util::rlp::{RlpStream, Rlp, Decodable};

#[bench]
fn bench_stream_value(b: &mut Bencher) {
    b.iter( || {
        //1029
        let mut stream = RlpStream::new();
        stream.append(&1029u32);
        let _ = stream.out().unwrap();
    });
}

#[bench]
fn bench_stream_nested_empty_lists(b: &mut Bencher) {
    b.iter( || {
        // [ [], [[]], [ [], [[]] ] ]
        let mut stream = RlpStream::new_list(3);
        stream.append_list(0);
        stream.append_list(1).append_list(0);
        stream.append_list(2).append_list(0).append_list(1).append_list(0);
        let _ = stream.out().unwrap();
    });
}

#[bench]
fn bench_decode_nested_empty_lists(b: &mut Bencher) {
    b.iter( || {
        // [ [], [[]], [ [], [[]] ] ]
        let data = vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0];
        let rlp = Rlp::new(&data);
        let v0: Vec<u8> = Decodable::decode(&rlp.at(0).unwrap()).unwrap();
        let v1: Vec<Vec<u8>> = Decodable::decode(&rlp.at(1).unwrap()).unwrap();
        let v2a: Vec<u8> = Decodable::decode(&rlp.at(2).unwrap().at(0).unwrap()).unwrap();
        let v2b: Vec<Vec<u8>> = Decodable::decode(&rlp.at(2).unwrap().at(1).unwrap()).unwrap();
    });
}

#[bench]
fn bench_stream_1000_empty_lists(b: &mut Bencher) {
    b.iter( || {
        let mut stream = RlpStream::new();
        for _ in 0..1000 {
            stream.append_list(0);
        }
        let _ = stream.out().unwrap();
    });
}
