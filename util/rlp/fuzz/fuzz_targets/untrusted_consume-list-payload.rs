#![no_main]
#![cfg(feature = "nightly")]
#[macro_use] extern crate libfuzzer_sys;
extern crate rlp;

use rlp::UntrustedRlp;

fuzz_target!(|data: &[u8]| {
    let dlen = data.len();
    let dvec = Vec::with_capacity(dlen + 1);

    // Initialize rlpstream data as a list
    dvec.push(0xc + dlen as u8);

    // Push data stream onto raw rlpstream data
    for d in data.iter() {
        dvec.push(d);
    }
    let urlp = UntrustedRlp::new(dvec.as_bytes());

    // Internally calls BasicDecoder::payload_info(self.bytes)
    // Which will hit the code-path we're interested in fuzzing
    let _ = urlp.data();
});
