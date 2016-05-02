#![warn(missing_docs)]
#![cfg_attr(feature="nightly", feature(custom_derive, custom_attribute, plugin))]
#![cfg_attr(feature="nightly", plugin(serde_macros, clippy))]

extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;

mod spec;
mod contract;
mod address;
mod token;
mod coders;
mod error;
