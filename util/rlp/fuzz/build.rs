extern crate rustc_version;
use rustc_version::{version, version_meta, Channel, Version};

fn main() {
    assert!(version().unwrap().major >= 1);

    match version_meta().unwrap().channel {
        Channel::Stable => {
        }
        Channel::Beta => {
        }
        Channel::Nightly => {
            println!("cargo:rustc-cfg=feature=\"nightly\"")
        }
        Channel::Dev => {
        }
    }
    
}
