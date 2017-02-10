#[cfg(feature = "docopt")]
include!("./ethbin.rs");

#[cfg(not(feature = "docopt"))]
fn main() {}

