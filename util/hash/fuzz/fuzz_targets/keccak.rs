#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate keccak_hash;
use keccak_hash::keccak;

fuzz_target!(|data: &[u8]| {
    keccak(data);
});
