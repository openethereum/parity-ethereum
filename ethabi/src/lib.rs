//! Ethereum ABI encoding decoding library.

#![warn(missing_docs)]
#![cfg_attr(feature="nightly", feature(custom_derive, custom_attribute, plugin))]
#![cfg_attr(feature="nightly", plugin(serde_macros, clippy))]

extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;

pub mod spec;
mod constructor;
mod contract;
mod decoder;
mod encoder;
mod error;
mod function;
mod token;

pub use self::constructor::Constructor;
pub use self::contract::Contract;
pub use self::token::Token;
pub use self::error::Error;
pub use self::encoder::Encoder;
pub use self::decoder::Decoder;
pub use self::function::Function;
