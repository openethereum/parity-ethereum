#![no_main]
#![cfg(feature = "nightly")]
#[macro_use] extern crate libfuzzer_sys;
extern crate rlp;

use rlp::UntrustedRlp;

fuzz_target!(|data: &[u8]| {
    // Create UntrustedRlp to build BasicDecoder
    let urlp = UntrustedRlp::new(data);

    // Internally calls BasicDecoder::payload_info(self.bytes)
    // Which will hit the code-path we're interested in fuzzing
    let _ = urlp.data();
});
