//! benchmarking for rlp
//! should be started with:
//! ```bash
//! multirust run nightly cargo bench 
//! ```

#![feature(test)]

extern crate test;
extern crate ethcore_util;

use test::Bencher;
use ethcore_util::rlp::{RlpStream};

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
fn bench_stream_1000_empty_lists(b: &mut Bencher) {
    b.iter( || {
        let mut stream = RlpStream::new();
        for _ in 0..1000 {
            stream.append_list(0);
        }
        let _ = stream.out().unwrap();
    });
}
