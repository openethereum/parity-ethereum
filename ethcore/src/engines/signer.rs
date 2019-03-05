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

//! A signer used by Engines which need to sign messages.

use ethereum_types::{H256, Address};
use ethkey::{self, Signature};
use ethkey::crypto::ecies;

/// Everything that an Engine needs to sign messages.
pub trait EngineSigner: Send + Sync {
	/// Sign a consensus message hash.
	fn sign(&self, hash: H256) -> Result<Signature, ethkey::Error>;

	/// Signing address
	fn address(&self) -> Address;

	/// Decrypt a message that was encrypted to this signer's key.
	fn decrypt(&self, auth_data: &[u8], cipher: &[u8]) -> Result<Vec<u8>, ethkey::crypto::Error>;

	/// The signer's public key, if available.
	fn public(&self) -> Option<ethkey::Public>;
}

/// Creates a new `EngineSigner` from given key pair.
pub fn from_keypair(keypair: ethkey::KeyPair) -> Box<EngineSigner> {
	Box::new(Signer(keypair))
}

struct Signer(ethkey::KeyPair);

impl EngineSigner for Signer {
	fn sign(&self, hash: H256) -> Result<Signature, ethkey::Error> {
		ethkey::sign(self.0.secret(), &hash)
	}

	fn decrypt(&self, auth_data: &[u8], cipher: &[u8]) -> Result<Vec<u8>, ethkey::crypto::Error> {
		ecies::decrypt(self.0.secret(), auth_data, cipher)
	}

	fn address(&self) -> Address {
		self.0.address()
	}

	fn public(&self) -> Option<ethkey::Public> {
		Some(*self.0.public())
	}
}

#[cfg(test)]
mod test_signer {
	use std::sync::Arc;

	use ethkey::Password;
	use accounts::{self, AccountProvider, SignError};

	use super::*;

	impl EngineSigner for (Arc<AccountProvider>, Address, Password) {
		fn sign(&self, hash: H256) -> Result<Signature, ethkey::Error> {
			match self.0.sign(self.1, Some(self.2.clone()), hash) {
				Err(SignError::NotUnlocked) => unreachable!(),
				Err(SignError::NotFound) => Err(ethkey::Error::InvalidAddress),
				Err(SignError::Hardware(err)) => {
					warn!("Error using hardware wallet for engine: {:?}", err);
					Err(ethkey::Error::InvalidSecret)
				},
				Err(SignError::SStore(accounts::Error::EthKey(err))) => Err(err),
				Err(SignError::SStore(accounts::Error::EthKeyCrypto(err))) => {
					warn!("Low level crypto error: {:?}", err);
					Err(ethkey::Error::InvalidSecret)
				},
				Err(SignError::SStore(err)) => {
					warn!("Error signing for engine: {:?}", err);
					Err(ethkey::Error::InvalidSignature)
				},
				Ok(ok) => Ok(ok),
			}
		}

		fn decrypt(&self, auth_data: &[u8], cipher: &[u8]) -> Result<Vec<u8>, ethkey::crypto::Error> {
			match self.0.decrypt(self.1, None, auth_data, cipher) {
				Ok(plain) => Ok(plain),
				Err(e) => {
					warn!("Unable to decrypt message: {:?}", e);
					Err(ethkey::crypto::Error::InvalidMessage)
				},
			}
		}

		fn address(&self) -> Address {
			self.1
		}

		fn public(&self) -> Option<ethkey::Public> {
			self.0.account_public(self.1, &self.2).ok()
		}
	}
}
