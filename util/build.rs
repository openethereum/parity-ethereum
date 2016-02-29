extern crate vergen;
extern crate rustc_version;

use vergen::*;
use rustc_version::{version_meta, Channel};

fn main() {
	vergen(OutputFns::all()).unwrap();

    if let Channel::Nightly = version_meta().channel {
		println!("cargo:rustc-cfg=x64asm");
    }
}
