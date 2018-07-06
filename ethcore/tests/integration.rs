#![cfg(feature = "json-tests")]

extern crate ethcore;
extern crate ethcore_transaction as transaction;
extern crate ethereum_types;
extern crate ethjson;
extern crate evm;
extern crate keccak_hasher;
extern crate memorydb;
extern crate patricia_trie as trie;
extern crate patricia_trie_ethereum as ethtrie;
extern crate rlp;
extern crate vm;
extern crate ethcore_logger;
extern crate ethcore_io as io;
extern crate tempdir;
extern crate ethcore_bytes as bytes;
extern crate keccak_hash as hash;

#[macro_use]
extern crate macros;

#[macro_use]
extern crate log;
mod json;
