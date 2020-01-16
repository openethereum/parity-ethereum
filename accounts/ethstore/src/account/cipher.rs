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

use json;

#[derive(Debug, PartialEq, Clone)]
pub struct Aes128Ctr {
	pub iv: [u8; 16],
}

#[derive(Debug, PartialEq, Clone)]
pub enum Cipher {
	Aes128Ctr(Aes128Ctr),
}

impl From<json::Aes128Ctr> for Aes128Ctr {
	fn from(json: json::Aes128Ctr) -> Self {
		Aes128Ctr {
			iv: json.iv.into()
		}
	}
}

impl Into<json::Aes128Ctr> for Aes128Ctr {
	fn into(self) -> json::Aes128Ctr {
		json::Aes128Ctr {
			iv: From::from(self.iv)
		}
	}
}

impl From<json::Cipher> for Cipher {
	fn from(json: json::Cipher) -> Self {
		match json {
			json::Cipher::Aes128Ctr(params) => Cipher::Aes128Ctr(From::from(params)),
		}
	}
}

impl Into<json::Cipher> for Cipher {
	fn into(self) -> json::Cipher {
		match self {
			Cipher::Aes128Ctr(params) => json::Cipher::Aes128Ctr(params.into()),
		}
	}
}
