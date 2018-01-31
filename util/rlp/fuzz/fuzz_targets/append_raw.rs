#![no_main]
#![cfg(feature = "nightly")]
#[macro_use] extern crate libfuzzer_sys;
extern crate rlp;
extern crate rand;

use rlp::RlpStream;
use rand::Rng;

fuzz_target!(|data: &[u8]| {
    let mut rls = RlpStream::new();

    // Generate random item count to further fuzz append_raw's parsing
    let mut rng = rand::thread_rng();
    let rand_item_count = rng.gen_range::<usize>(1 as usize, 4096 as usize);

    let _ = rls.append_raw(data, rand_item_count);
});

