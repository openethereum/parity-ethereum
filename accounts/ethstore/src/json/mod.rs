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

//! Contract interface specification.

mod bytes;
mod cipher;
mod crypto;
mod error;
mod hash;
mod id;
mod kdf;
mod key_file;
mod presale;
mod vault_file;
mod vault_key_file;
mod version;

pub use self::{
    bytes::Bytes,
    cipher::{Aes128Ctr, Cipher, CipherSer, CipherSerParams},
    crypto::{CipherText, Crypto},
    error::Error,
    hash::{H128, H160, H256},
    id::Uuid,
    kdf::{Kdf, KdfSer, KdfSerParams, Pbkdf2, Prf, Scrypt},
    key_file::{KeyFile, OpaqueKeyFile},
    presale::{Encseed, PresaleWallet},
    vault_file::VaultFile,
    vault_key_file::{
        insert_vault_name_to_json_meta, remove_vault_name_from_json_meta, VaultKeyFile,
        VaultKeyMeta,
    },
    version::Version,
};
