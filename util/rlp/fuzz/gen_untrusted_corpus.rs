extern crate rlp;
extern crate ethcore_bigint;

use std::env;
use std::error::Error;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use rlp::RlpStream;
use ethcore_bigint::prelude::{U128, U256, H64, H128, H160, H256, H512, H520, H2048};

fn create_file(path: &Path) -> File {
    match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           path.display(),
                           why.description()),
        Ok(file) => file
    } 
}

fn write_to_file(f: &File, rlp: RlpStream) {
    let mut g = f.clone();
    match g.write_all(rlp.as_raw()) {
        Err(why) => {
            panic!("couldn't write to file: {}", why.description())
        },
        Ok(_) => println!("successfully wrote to file")
    }

}

fn create_uint_stream() {
    // Create RLP Stream to encode values
    let mut rlp = RlpStream::new();
    // U8, U16, U32, U64, U128, U256 bytes 
    let u8b = 8 as u8;
    let u16b = 16 as u16;
    let u32b = 32 as u32;
    let u64b = 64 as u64;
    let u128b = U128::from(128);
    let u256b = U256::from(256);

    rlp.append(&u8b);
    rlp.append(&u16b);
    rlp.append(&u32b);
    rlp.append(&u64b);
    rlp.append(&u128b);
    rlp.append(&u256b);

    // Read in base path to fuzzing corpus directory from `RLPCORPUS` environment var
    let corp = env::var("RLPCORPUS").unwrap();
    let fp = format!("{}{}", corp, "/untrusted_data/well-formed-list-uint");
    let path = Path::new(&fp);

    // Write RLP Stream to corpus file
    let f = create_file(&path);
    write_to_file(&f, rlp);
}

fn create_hash_stream() {
    // Create RLP Stream to encode values
    let mut rlp = RlpStream::new();
    // H64, H128, H160, H256, H512, H520, H2048 list
    let h64 = H64::random();
    let h128 = H128::random();
    let h160 = H160::random();
    let h256 = H256::random();
    let h512 = H512::random();
    let h520 = H520::random();
    let h2048 = H2048::random();


    rlp.append(&h64);
    rlp.append(&h128);
    rlp.append(&h160);
    rlp.append(&h256);
    rlp.append(&h512);
    rlp.append(&h520);
    rlp.append(&h2048);

    // Read in base path to fuzzing corpus directory from `RLPCORPUS` environment var
    let corp = env::var("RLPCORPUS").unwrap();
    let fp = format!("{}{}", corp, "/untrusted_data/well-formed-list-hash");
    let path = Path::new(&fp);

    // Write RLP Stream to corpus file
    let f = create_file(&path);
    write_to_file(&f, rlp);
}

fn main() {
    create_uint_stream();
    create_hash_stream();
}
