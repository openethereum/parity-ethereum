#![no_main]
#![cfg(feature = "nightly")]
#[macro_use] extern crate libfuzzer_sys;
extern crate rlp;
extern crate ethcore_bigint;

use rlp::{DecoderError, UntrustedRlp};
use ethcore_bigint::prelude::{U128, U256, H64, H128, H160, H256, H512, H520, H2048};

fuzz_target!(|data: &[u8]| {
    // Create UntrustedRlp to build BasicDecoder
    let urlp = UntrustedRlp::new(&data);

    // Attempt to create panic by decoding 
    let _: Result<u8, DecoderError> = urlp.as_val();
    let _: Result<u16, DecoderError> = urlp.as_val();
    let _: Result<u32, DecoderError> = urlp.as_val();
    let _: Result<u64, DecoderError> = urlp.as_val();
    let _: Result<U128, DecoderError> = urlp.as_val();
    let _: Result<U256, DecoderError> = urlp.as_val();

    let _: Result<H64, DecoderError> = urlp.as_val();
    let _: Result<H128, DecoderError> = urlp.as_val();
    let _: Result<H160, DecoderError> = urlp.as_val();
    let _: Result<H256, DecoderError> = urlp.as_val();
    let _: Result<H512, DecoderError> = urlp.as_val();
    let _: Result<H520, DecoderError> = urlp.as_val();
    let _: Result<H2048, DecoderError> = urlp.as_val();

    let _: Result<Vec<u8>, DecoderError> = urlp.as_val();
    let _: Result<Vec<u8>, DecoderError> = urlp.as_list();
});
