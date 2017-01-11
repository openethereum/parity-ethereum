// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use secp256k1::key;
use bigint::hash::H256;
use {Error};

#[derive(Clone, PartialEq, Eq)]
pub struct Secret {
	inner: H256,
}

impl fmt::Debug for Secret {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "Secret: 0x{:x}{:x}..{:x}{:x}", self.inner[0], self.inner[1], self.inner[30], self.inner[31])
	}
}

impl Secret {
	pub fn from_slice(key: &[u8]) -> Result<Self, Error> {
		if key.len() != 32 {
			return Err(Error::InvalidSecret);
		}

		let mut h = H256::default();
		h.copy_from_slice(&key[0..32]);
		Ok(Secret { inner: h })
	}
}

impl FromStr for Secret {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let hash = H256::from_str(s).map_err(|e| Error::Custom(format!("{:?}", e)))?;
		Self::from_slice(&hash)
	}
}

impl From<key::SecretKey> for Secret {
	fn from(key: key::SecretKey) -> Self {
		Self::from_slice(&key[0..32])
			.expect("`key::SecretKey` is valid (no way to construct invalid one); qed")
	}
}

impl Deref for Secret {
	type Target = H256;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

