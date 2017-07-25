#![no_main]
#[macro_use] extern crate libfuzzer_sys;
#[cfg(feature = "nightly"])
extern crate rlp;

use rlp::UntrustedRlp;

fuzz_target!(|data: &[u8]| {
    let _ = UntrustedRlp::new(data);
});
