#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate keccak_hash;
use keccak_hash::{keccak, keccak_256, keccak_512};

fuzz_target!(|data: &[u8]| {
    keccak(data);
    unsafe {
        let mut data_m: Vec<u8> = Vec::with_capacity(data.len());
        data_m.extend_from_slice(data);
        keccak_256(data_m.as_mut_slice().as_mut_ptr(), data_m.len(), data.as_ptr(), data.len());
        keccak_512(data_m.as_mut_slice().as_mut_ptr(), data_m.len(), data.as_ptr(), data.len());
    }
});
