//! Ethereum ABI encoding decoding library.

#![warn(missing_docs)]
#![cfg_attr(feature="nightly", feature(plugin))]
#![cfg_attr(feature="nightly", plugin(clippy))]

extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate tiny_keccak;

#[macro_use]
extern crate serde_derive;

pub mod spec;
pub mod token;
mod constructor;
mod contract;
mod decoder;
mod encoder;
mod error;
mod function;
mod event;
mod signature;
pub mod util;

pub use self::spec::Interface;
pub use self::constructor::Constructor;
pub use self::contract::Contract;
pub use self::token::Token;
pub use self::error::Error;
pub use self::encoder::Encoder;
pub use self::decoder::Decoder;
pub use self::function::Function;
pub use self::event::Event;

/// ABI address.
pub type Address = [u8; 20];

/// ABI unsigned integer.
pub type Uint = [u8; 32];

/// ABI signed integer.
pub type Int = [u8; 32];
