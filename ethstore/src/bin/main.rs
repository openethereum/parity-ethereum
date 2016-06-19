#[cfg(feature = "cli")]
include!("ethstore.rs");

#[cfg(not(feature = "cli"))]
fn main() {}
