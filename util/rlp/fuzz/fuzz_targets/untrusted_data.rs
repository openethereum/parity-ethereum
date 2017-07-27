#![no_main]
#![cfg(feature = "nightly")]
#[macro_use] extern crate libfuzzer_sys;
extern crate rlp;

use rlp::UntrustedRlp;

fn iter_recursive(rlp: UntrustedRlp) {
    for x in rlp.iter() {
        // Internally calls BasicDecoder::payload_info(self.bytes)
        // Which will hit the code-path we're interested in fuzzing
        let _ = x.data();
        iter_recursive(x);
    }
}

fuzz_target!(|data: &[u8]| {
    // Create UntrustedRlp to build BasicDecoder
    let urlp = UntrustedRlp::new(data);
    // Attempt to recursively iterate over entire RLP structure
    iter_recursive(urlp);
});
