#[cfg(feature = "cli")]
include!("ethkey.rs");

#[cfg(not(feature = "cli"))]
fn main() {}
