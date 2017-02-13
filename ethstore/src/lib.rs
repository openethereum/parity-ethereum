// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

extern crate libc;
extern crate itertools;
extern crate smallvec;
extern crate rand;
extern crate time;
extern crate serde;
extern crate serde_json;
extern crate rustc_serialize;
extern crate crypto as rcrypto;
extern crate tiny_keccak;
extern crate parking_lot;
extern crate ethcore_devtools as devtools;

// reexport it nicely
extern crate ethkey as _ethkey;
extern crate ethcrypto as crypto;
extern crate ethcore_util as util;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

pub mod dir;
pub mod ethkey;

mod account;
mod json;

mod error;
mod ethstore;
mod import;
mod presale;
mod random;
mod secret_store;

pub use self::account::{SafeAccount};
pub use self::error::Error;
pub use self::ethstore::{EthStore, EthMultiStore};
pub use self::import::{import_accounts, read_geth_accounts};
pub use self::presale::PresaleWallet;
pub use self::secret_store::{
	SecretVaultRef, StoreAccountRef, SimpleSecretStore, SecretStore,
	Derivation, IndexDerivation,
};
pub use self::random::{random_phrase, random_string};
