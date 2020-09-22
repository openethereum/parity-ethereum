// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Ethereum key-management.

#![warn(missing_docs)]

extern crate dir;
extern crate itertools;
extern crate libc;
extern crate parking_lot;
extern crate rand;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
extern crate smallvec;
extern crate tempdir;
extern crate time;
extern crate tiny_keccak;

extern crate ethereum_types;
extern crate ethkey as _ethkey;
extern crate parity_crypto as crypto;
extern crate parity_wordlist;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate matches;

pub mod accounts_dir;
pub mod ethkey;

mod account;
mod json;

mod error;
mod ethstore;
mod import;
mod presale;
mod random;
mod secret_store;

pub use self::{
    account::{Crypto, SafeAccount},
    error::Error,
    ethstore::{EthMultiStore, EthStore},
    import::{import_account, import_accounts},
    json::OpaqueKeyFile as KeyFile,
    parity_wordlist::random_phrase,
    presale::PresaleWallet,
    random::random_string,
    secret_store::{
        Derivation, IndexDerivation, SecretStore, SecretVaultRef, SimpleSecretStore,
        StoreAccountRef,
    },
};

/// An opaque wrapper for secret.
pub struct OpaqueSecret(::ethkey::Secret);
