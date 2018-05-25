//! Ethereum blooms database
//!
//! zero allocation
//! zero copying

extern crate byteorder;
extern crate ethbloom;
extern crate tiny_keccak;

#[cfg(test)]
extern crate tempdir;

mod db;
mod file;
mod meta;
mod pending;

pub const VERSION: u64 = 1;

pub use db::{Database, DatabaseIterator};
