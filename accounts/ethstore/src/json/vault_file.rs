// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::io::{Read, Write};
use serde_json;
use super::Crypto;

/// Vault meta file
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VaultFile {
	/// Vault password, encrypted with vault password
	pub crypto: Crypto,
	/// Vault metadata string
	pub meta: Option<String>,
}

impl VaultFile {
	pub fn load<R>(reader: R) -> Result<Self, serde_json::Error> where R: Read {
		serde_json::from_reader(reader)
	}

	pub fn write<W>(&self, writer: &mut W) -> Result<(), serde_json::Error> where W: Write {
		serde_json::to_writer(writer, self)
	}
}

#[cfg(test)]
mod test {
	use serde_json;
	use json::{VaultFile, Crypto, Cipher, Aes128Ctr, Kdf, Pbkdf2, Prf};

	#[test]
	fn to_and_from_json() {
		let file = VaultFile {
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: "0155e3690be19fbfbecabcd440aa284b".into(),
				}),
				ciphertext: "4d6938a1f49b7782".into(),
				kdf: Kdf::Pbkdf2(Pbkdf2 {
					c: 1024,
					dklen: 32,
					prf: Prf::HmacSha256,
					salt: "b6a9338a7ccd39288a86dba73bfecd9101b4f3db9c9830e7c76afdbd4f6872e5".into(),
				}),
				mac: "16381463ea11c6eb2239a9f339c2e780516d29d234ce30ac5f166f9080b5a262".into(),
			},
			meta: Some("{}".into()),
		};

		let serialized = serde_json::to_string(&file).unwrap();
		let deserialized = serde_json::from_str(&serialized).unwrap();

		assert_eq!(file, deserialized);
	}

	#[test]
	fn to_and_from_json_no_meta() {
		let file = VaultFile {
			crypto: Crypto {
				cipher: Cipher::Aes128Ctr(Aes128Ctr {
					iv: "0155e3690be19fbfbecabcd440aa284b".into(),
				}),
				ciphertext: "4d6938a1f49b7782".into(),
				kdf: Kdf::Pbkdf2(Pbkdf2 {
					c: 1024,
					dklen: 32,
					prf: Prf::HmacSha256,
					salt: "b6a9338a7ccd39288a86dba73bfecd9101b4f3db9c9830e7c76afdbd4f6872e5".into(),
				}),
				mac: "16381463ea11c6eb2239a9f339c2e780516d29d234ce30ac5f166f9080b5a262".into(),
			},
			meta: None,
		};

		let serialized = serde_json::to_string(&file).unwrap();
		let deserialized = serde_json::from_str(&serialized).unwrap();

		assert_eq!(file, deserialized);
	}
}
