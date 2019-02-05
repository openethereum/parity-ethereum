// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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
pub enum Prf {
	HmacSha256,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pbkdf2 {
	pub c: u32,
	pub dklen: u32,
	pub prf: Prf,
	pub salt: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Scrypt {
	pub dklen: u32,
	pub p: u32,
	pub n: u32,
	pub r: u32,
	pub salt: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Kdf {
	Pbkdf2(Pbkdf2),
	Scrypt(Scrypt),
}

impl From<json::Prf> for Prf {
	fn from(json: json::Prf) -> Self {
		match json {
			json::Prf::HmacSha256 => Prf::HmacSha256,
		}
	}
}

impl Into<json::Prf> for Prf {
	fn into(self) -> json::Prf {
		match self {
			Prf::HmacSha256 => json::Prf::HmacSha256,
		}
	}
}

impl From<json::Pbkdf2> for Pbkdf2 {
	fn from(json: json::Pbkdf2) -> Self {
		Pbkdf2 {
			c: json.c,
			dklen: json.dklen,
			prf: From::from(json.prf),
			salt: json.salt.into(),
		}
	}
}

impl Into<json::Pbkdf2> for Pbkdf2 {
	fn into(self) -> json::Pbkdf2 {
		json::Pbkdf2 {
			c: self.c,
			dklen: self.dklen,
			prf: self.prf.into(),
			salt: From::from(self.salt),
		}
	}
}

impl From<json::Scrypt> for Scrypt {
	fn from(json: json::Scrypt) -> Self {
		Scrypt {
			dklen: json.dklen,
			p: json.p,
			n: json.n,
			r: json.r,
			salt: json.salt.into(),
		}
	}
}

impl Into<json::Scrypt> for Scrypt {
	fn into(self) -> json::Scrypt {
		json::Scrypt {
			dklen: self.dklen,
			p: self.p,
			n: self.n,
			r: self.r,
			salt: From::from(self.salt),
		}
	}
}

impl From<json::Kdf> for Kdf {
	fn from(json: json::Kdf) -> Self {
		match json {
			json::Kdf::Pbkdf2(params) => Kdf::Pbkdf2(From::from(params)),
			json::Kdf::Scrypt(params) => Kdf::Scrypt(From::from(params)),
		}
	}
}

impl Into<json::Kdf> for Kdf {
	fn into(self) -> json::Kdf {
		match self {
			Kdf::Pbkdf2(params) => json::Kdf::Pbkdf2(params.into()),
			Kdf::Scrypt(params) => json::Kdf::Scrypt(params.into()),
		}
	}
}
