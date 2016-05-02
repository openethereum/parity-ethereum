
#[cfg(feature = "serde_macros")]
include!("mod.rs.in");

#[cfg(not(feature = "serde_macros"))]
include!(concat!(env!("OUT_DIR"), "/mod.rs"));

